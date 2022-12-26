pub mod matrix {
    use std::fmt::Error;

    use nix::{
        sys::wait::{WaitPidFlag, WaitStatus},
        unistd::{fork, ForkResult},
    };

    pub fn print_matrix(matrix: &Vec<Vec<i64>>) {
        for row in matrix {
            println!("{:?}", row);
        }
    }

    pub fn calculate_determinant(
        matrix: &Vec<Vec<i64>>,
        coefficient_arg: i64,
        is_positive_arg: bool,
    ) -> i64 {
        if matrix.len() > 1 {
            let mut calculated = Vec::with_capacity(matrix.len());
            for (ind, row) in matrix.iter().enumerate() {
                let stripped = select_minor(matrix, ind, 0);
                let is_positive = ind % 2 == 0;
                let coefficient = row[0];
                calculated.push(match is_positive_arg {
                    true => {
                        coefficient_arg * calculate_determinant(&stripped, coefficient, is_positive)
                    }
                    false => {
                        -coefficient_arg
                            * calculate_determinant(&stripped, coefficient, is_positive)
                    }
                })
                // return calculate_determinant(&stripped, coefficient, is_positive);
            }
            return calculated.iter().sum();
        } else {
            return match is_positive_arg {
                true => matrix[0][0] * coefficient_arg,
                false => matrix[0][0] * -coefficient_arg,
            };
        }
    }

    use std::io::{Read, Write};
    use std::os::unix::net::{UnixListener, UnixStream};

    
    fn calculate_matrix_slice_and_write_socket(
        matrix: &Vec<Vec<i64>>,
        socket_path: &str,
        start: usize,
        end: usize,
    ) {
        let data = calculate_slice(matrix, start, end);

        let mut unix_stream =
            UnixStream::connect(socket_path).expect("Failed at creating unix stream");

        write_request_and_shutdown(&mut unix_stream, &data.to_ne_bytes())
    }

    fn write_request_and_shutdown(unix_stream: &mut UnixStream, data: &[u8]) {
        unix_stream
            .write(data)
            .expect("Failed at writing onto the stream");

        println!("Request sent");
        println!("Shutting down writing on the stream, waiting for response...");

        unix_stream
            .shutdown(std::net::Shutdown::Write)
            .expect("Could not shutdown writing on the stream");
    }

    pub fn calculate_slice(matrix: &Vec<Vec<i64>>, start: usize, end: usize) -> i64 {
        assert!(start < end);
        let mut calculated = Vec::with_capacity(end - start);

        for i in start..end {
            let stripped = select_minor(matrix, 0, i);
            let coefficient = matrix[0][i];
            let is_positive = i % 2 == 0;
            calculated.push(calculate_determinant(&stripped, coefficient, is_positive));
        }

        return calculated.iter().sum();
    }

    pub unsafe fn socket_calculate_determinant(matrix: &Vec<Vec<i64>>, socket_path: &str) {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => match unsafe { fork() } {
                Ok(ForkResult::Parent { child }) => (),
                Ok(ForkResult::Child) => calculate_matrix_slice_and_write_socket(
                    matrix,
                    socket_path,
                    matrix.len()/2,
                    matrix.len(),
                ),
                Err(_) => todo!(),
            },
            Ok(ForkResult::Child) => {
                calculate_matrix_slice_and_write_socket(matrix, socket_path, 0, matrix.len() / 2)
            }
            Err(_) => todo!(),
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
        matrix: &Vec<Vec<i64>>,
        out_shm_ptr: *mut u8,
        coefficient: i64,
        is_positive: bool,
    ) {
        // std::thread::sleep(std::time::Duration::from_millis(100));

        // let vec_ptr = read_usize_from_shm(out_shm_ptr) as *const Vec<i64>;

        // let matrix = vec_ptr_to_matrix(vec_ptr);

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
                    let read = read_usize_from_shm(out_shm_ptr) as i64;
                    let module = result * coefficient;
                    let write = if is_positive {
                        read + module
                    } else {
                        read - module
                    };
                    write_usize_to_shm(out_shm_ptr, write as usize);
                    // return;
                }
                Ok(ForkResult::Child) => {
                    println!("Child matrix {:?}", matrix);
                    for (ind, row) in matrix.iter().enumerate() {
                        let stripped = select_minor(&matrix, ind, 0);
                        println!("Stripped is: {:?} by col {}", stripped, ind);
                        let is_positive = ind as i64 % 2 == 0;
                        let coef = row[0];
                        // write_usize_to_shm(shmem.as_ptr(), stripped.as_ptr() as usize);
                        shm_determinant_calculation(&stripped, shmem.as_ptr(), coef, is_positive);
                    }
                    while let Ok(WaitStatus::StillAlive) = nix::sys::wait::wait() {}
                    std::process::exit(0);
                }
                Err(_) => println!("Fork failed!"),
            }
        }
        if matrix.len() == 1 {
            let read = read_usize_from_shm(out_shm_ptr) as i64;
            let module = matrix[0][0] * coefficient;
            let write = if is_positive {
                module + read
            } else {
                -module + read
            };
            println!("\tRead {} from shared memory", read);
            println!("\tDet is {}", module);
            println!("\tWrite {} to shared memory", write);
            match is_positive {
                true => write_usize_to_shm(out_shm_ptr, write as usize),
                false => write_usize_to_shm(out_shm_ptr, write as usize),
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

    use std::fs::File;
    use std::io::{BufRead, BufReader};
    pub fn read_matrix(path: &str) -> Vec<Vec<i64>> {
        let mut f = BufReader::new(File::open(path).unwrap());

        let arr: Vec<Vec<i64>> = f
            .lines()
            .map(|l| {
                l.unwrap()
                    .split(char::is_whitespace)
                    .map(|number| number.parse().unwrap())
                    .collect()
            })
            .collect();

        return arr;
    }
}

#[cfg(test)]
mod tests {

    use crate::matrix::*;

    use super::*;

    const TEST_DATA_PATH: &str = "src/test_data";

    // const matrix: [[i64; 2]; 2] = [[1, 2], [3, 4]];

    // const matrix_1: (Matrix, Determinant)     = ([[4, 7, 9, 5], [3, 6, 9, 4], [0, 4, 26, 8], [9, 6, 3, 7]], -282);

    // const matrix_: [[i64; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

    fn get_test_data() -> Vec<Vec<Vec<i64>>> {
        return vec![
            read_matrix(&format!("{}/-282.txt", TEST_DATA_PATH)),
            read_matrix(&format!("{}/0a.txt", TEST_DATA_PATH)),
            read_matrix(&format!("{}/0b.txt", TEST_DATA_PATH)),
            read_matrix(&format!("{}/0c.txt", TEST_DATA_PATH)),
        ];
    }

    #[test]
    fn test_slice_calculation() {
        let test_data = get_test_data();

        for matrix in test_data {
            let first = calculate_slice(&matrix, 0, matrix.len() / 2);
            let second = calculate_slice(&matrix, matrix.len() / 2, matrix.len());
            let whole = calculate_determinant(&matrix, 1, true);

            assert_eq!(first + second, whole);
        }
    }

    #[test]
    fn test_reading_matrix_from_file() {
        let reference: Vec<Vec<i64>> = vec![
            vec![4, 7, 9, 5],
            vec![3, 6, 9, 4],
            vec![0, 4, 26, 8],
            vec![9, 6, 3, 7],
        ];

        let read = read_matrix("src/test_data/-282.txt");

        print_matrix(&read);

        for (refer, result) in reference.iter().zip(read) {
            for (i, j) in refer.iter().zip(result) {
                assert_eq!(*i, j);
            }
        }
    }

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
