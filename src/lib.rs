pub mod matrix {
    use std::fmt::Error;

    use nix::{
        sys::wait::{WaitPidFlag, WaitStatus},
        unistd::{fork, ForkResult},
    };

    #[derive(Debug)]
    pub enum MatrixCalcError {
        NonSquare,
    }

    pub fn print_matrix(matrix: &Vec<Vec<i64>>) {
        for row in matrix {
            println!("{:?}", row);
        }
    }

    pub fn calculate_determinant(
        matrix: &Vec<Vec<i64>>,
        exclude_row: Option<usize>,
        _exclude_col: Option<usize>,
    ) -> Result<i64, MatrixCalcError> {
        if matrix.len() != matrix[0].len() {
            return Err(MatrixCalcError::NonSquare);
        }

        if matrix.len() > 2 {
            match exclude_row {
                None => calculate_determinant(matrix, Some(0), Some(0)),
                Some(row) => {
                    if row >= matrix.len() {
                        Ok(0)
                    } else {
                        let stripped_matrix = select_minor(matrix, row, 0);
                        let sign = ((row as i64 % 2) * -2) + 1;
                        let coef = matrix[row][0];
                        let deeper_det =
                            calculate_determinant(&stripped_matrix, Some(0), _exclude_col)
                                .expect("msg");
                        // let next_det = calculate_determinant(matrix, Some(row + 1), _exclude_col)
                        //     .expect("msg");
                        Ok(sign * coef * deeper_det)
                    }
                }
            }
        } else {
            let result = matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
            Ok(result)
        }
    }

    pub unsafe fn write_usize_to_shm(shm_ptr: *mut u8, value: usize) {
        let shift: usize = 256;
        for i in 1..=8_usize {
            let byte: u8 = (value / shift.pow(8 - i as u32)) as u8;
            // *shm_ptr.add(i-1) = byte;
            std::ptr::write_volatile(shm_ptr.add(i - 1), byte);
        }
    }

    pub unsafe fn read_usize_from_shm(shm_ptr: *const u8) -> usize {
        let mut ptr_accum: usize = 0;
        let shift: usize = 256;

        for i in 1..=8 {
            let byte: usize = std::ptr::read_volatile(shm_ptr.add(i - 1)) as usize;
            // println!("readed byte: {:x}", byte);
            // println!("shifted byte: {:x}", byte * shift.pow(8-i as u32));
            ptr_accum += byte * shift.pow(8 - i as u32);
        }
        ptr_accum
    }

    pub unsafe fn vec_ptr_to_matrix(ptr: *const Vec<i64>) -> Vec<Vec<i64>> {
        let mut vector = Vec::new();

        for i in 0..(*ptr).len() {
            vector.push((*ptr.add(i)).clone());
        }
        vector
    }

    pub unsafe fn shm_determinant_calculation(
        // matrix: &Vec<Vec<i64>>,
        out_shm_ptr: *mut u8,
        coefficient: i64,
        is_positive: bool,
    ) {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let vec_ptr = read_usize_from_shm(out_shm_ptr) as *const Vec<i64>;

        let matrix = vec_ptr_to_matrix(vec_ptr);

        
        if matrix.len() > 1 {
            let shmem = shared_memory::ShmemConf::new()
                .size(8)
                .create()
                .expect("Shmem create failed");
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => {
                    while let Ok(WaitStatus::StillAlive) = nix::sys::wait::waitpid(child, None) {}
                    let result = read_usize_from_shm(shmem.as_ptr()) as i64;
                    println!("\tGot det {} for {:?}", result, matrix);
                    write_usize_to_shm(
                        out_shm_ptr,
                        if is_positive {
                            (result as i64 * coefficient) as usize
                        } else {
                            (result as i64 * -coefficient) as usize
                        },
                    );
                    // return;
                }
                Ok(ForkResult::Child) => {
                    println!("Child matrix {:?}", matrix);
                    for (col, val) in matrix.iter().enumerate() {
                        let stripped = select_minor(&matrix, 0, col);
                        println!("Stripped is: {:?} by col {}", stripped, col);
                        let is_positive = col as i64 % 2 == 0;
                        let coef = matrix[0][col];
                        write_usize_to_shm(shmem.as_ptr(), stripped.as_ptr() as usize);
                        shm_determinant_calculation(shmem.as_ptr(), coef, is_positive);
                    }
                    while let Ok(WaitStatus::StillAlive) = nix::sys::wait::wait() {}
                    std::process::exit(0);
                }
                Err(_) => println!("Fork failed!"),
            }
        }
        if matrix.len() == 1 {
            match is_positive {
                true => write_usize_to_shm(out_shm_ptr, (matrix[0][0] * coefficient) as usize),
                false => write_usize_to_shm(out_shm_ptr, (matrix[0][0] * -coefficient) as usize),
            }
            // std::process::exit(0);
        }
    }

    use ipc_channel::ipc::TryRecvError;
    pub fn channel_determinant_calculation(
        matrix: &Vec<Vec<i64>>,
        // sender, через который будут отправляться результаты
        out_transmitter: ipc_channel::ipc::IpcSender<i64>,
        coefficient: i64,
        is_positive: bool,
        verbose: bool,
    ) {
        // проверка на квадратность матрицы
        if matrix.len() != matrix[0].len() {
            println!("Matrix is not square");
            return;
        }

        // локальный канал, для связи рекурсивных вызовов
        let (tx, rx) = ipc_channel::ipc::channel().expect("could not create ipc channel");

        if matrix.len() > 1 {
            // форк процесса
            match unsafe { nix::unistd::fork() } {
                // родительный процесс обрабатывает ресивер
                Ok(ForkResult::Parent { child }) => {
                    // убрать tx из текущей области видимости, чтобы не было бесполезных трансмиттеров
                    drop(tx);
                    loop {
                        match rx.try_recv() {
                            Ok(res) => {
                                if verbose {
                                    println!("Received {res} with {:?} from PID {child}", matrix);
                                }
                                out_transmitter
                                    .send(if is_positive {
                                        res * coefficient
                                    } else {
                                        res * (-coefficient)
                                    })
                                    .expect("Could not send data to passed transmitter");
                            }
                            Err(TryRecvError::Empty) => {
                                // задержка для наглядности процесса
                                if verbose {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                            }
                            Err(TryRecvError::IpcError(err)) => match err {
                                ipc_channel::ipc::IpcError::Bincode(_) => println!("Bincode error"),
                                ipc_channel::ipc::IpcError::Io(_) => println!("IO error"),
                                ipc_channel::ipc::IpcError::Disconnected => {
                                    if verbose {
                                        println!("Receiver disconnected");
                                    }
                                    break;
                                }
                            },
                        }
                    }
                }
                // дочерний процесс выделяет минор и рекурсивно вызывает эту же функцию
                Ok(ForkResult::Child) => {
                    for (col, val) in matrix.iter().enumerate() {
                        let stripped = select_minor(matrix, 0, col);
                        let is_positive = col as i64 % 2 == 0;
                        let coef = matrix[0][col];
                        channel_determinant_calculation(
                            &stripped,
                            tx.clone(),
                            coef,
                            is_positive,
                            verbose,
                        );
                    }
                    drop(tx);
                    std::process::exit(0)
                }
                Err(_) => println!("Fork failed!"),
            }
        }

        // когда матрица из одного элемента, тривиально высчитывается детерминант и умножается на коэффициент
        if matrix.len() == 1 {
            out_transmitter
                .send(if is_positive {
                    matrix[0][0] * coefficient
                } else {
                    -matrix[0][0] * coefficient
                })
                .expect("Failed at sending trivial determinant");
        }
    }

    pub fn select_minor(matrix: &Vec<Vec<i64>>, row: usize, col: usize) -> Vec<Vec<i64>> {
        let mut new_matrix: Vec<Vec<i64>> = Vec::new();

        for r in 0..matrix.len() {
            if r == row {
                continue;
            }
            let mut current_row: Vec<i64> = Vec::new();

            for c in 0..matrix[0].len() {
                if c == col {
                    continue;
                }
                current_row.push(matrix[r][c]);
            }

            new_matrix.push(current_row);
        }
        new_matrix
    }
}

#[cfg(test)]
mod tests {

    use crate::matrix::{read_usize_from_shm, write_usize_to_shm};

    use super::*;

    #[test]
    fn test_minor() {
        let v1 = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

        let sv1 = matrix::select_minor(&v1, 0, 0);
        let sv2 = matrix::select_minor(&v1, 0, 1);
        let sv3 = matrix::select_minor(&v1, 1, 0);
        let sv4 = matrix::select_minor(&v1, 1, 1);

        assert!(sv1.eq(&vec![vec![5, 6], vec![8, 9]]),);
        assert!(sv2.eq(&vec![vec![4, 6], vec![7, 9]]),);
        assert!(sv3.eq(&vec![vec![2, 3], vec![8, 9]]),);
        assert!(sv4.eq(&vec![vec![1, 3], vec![7, 9]]),);
    }

    #[test]
    fn test_read_write_shm() {
        let shm = shared_memory::ShmemConf::new()
            .size(8)
            .create()
            .expect("Failed at creating Shmem");

        let shm_ptr = shm.as_ptr();

        let original_value: usize = 0x123456789abcdef0;

        unsafe {
            write_usize_to_shm(shm_ptr, original_value);
            let read_value = read_usize_from_shm(shm_ptr);

            assert_eq!(original_value, read_value)
        }
    }
}
