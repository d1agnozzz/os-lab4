pub mod matrix {
    use std::fmt::Error;

    use nix::unistd::ForkResult;

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

    use ipc_channel::ipc::TryRecvError;

    pub fn interprocess_determinant_calculation(
        matrix: &Vec<Vec<i64>>,
        // sender, через который будут отправляться результаты
        transmitter: ipc_channel::ipc::IpcSender<i64>,
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
                                transmitter
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
                        interprocess_determinant_calculation(
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
            transmitter
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

    use super::*;

    #[test]
    fn test_subtraction() {
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
    fn test_determinant() {
        let v1 = vec![vec![1, 2], vec![3, 4]];
        let v2 = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

        let v3 = vec![
            vec![4, 7, 9, 5],
            vec![3, 6, 9, 4],
            vec![0, 4, 26, 8],
            vec![9, 6, 3, 7],
        ];
        let v4 = vec![
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 10],
            vec![11, 12, 13, 14, 15],
            vec![16, 17, 18, 19, 20],
            vec![21, 22, 23, 24, 25],
        ];
        let v5 = vec![
            vec![43, 76, -65, -12],
            vec![22, -87, 99, -22],
            vec![-76, 89, 57, -43],
            vec![43, -75, -62, 18],
        ];

        assert_eq!(matrix::calculate_determinant(&v1, None, None).unwrap(), -2);
        assert_eq!(matrix::calculate_determinant(&v2, None, None).unwrap(), 0);
        assert_eq!(
            matrix::calculate_determinant(&v3, None, None).unwrap(),
            -282
        );
        assert_eq!(matrix::calculate_determinant(&v4, None, None).unwrap(), 0);
        assert_eq!(
            matrix::calculate_determinant(&v5, None, None).unwrap(),
            30898426
        );
    }
}
