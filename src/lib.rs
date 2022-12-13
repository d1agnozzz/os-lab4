pub mod matrix {
    #[derive(Debug)]
    pub enum MatrixCalcError {
        NonSquare,
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
            // return match exclude_row {
            //     None => calculate_determinant(matrix, Some(0), Some(0)),
            //     Some(val) => {
            //         if val >= matrix.len() {
            //             return Ok(val as i64);
            //         } else {
            //             let stripped_matrix = subtract_row_and_col(matrix, exclude_row.unwrap(), 0);
            //             println!("---------------------------------------------");
            //             let sign = ((exclude_row.unwrap() as i64 % 2) * -2) + 1;
            //             let coef = matrix[exclude_row.unwrap()][0];
            //             let cur = calculate_determinant(&stripped_matrix, Some(0), _exclude_col)
            //                 .expect("msg");
            //             let next = calculate_determinant(
            //                 matrix,
            //                 Some(exclude_row.unwrap() + 1),
            //                 _exclude_col,
            //             )
            //             .expect("msg");
            //             println!("{} * {} * {} + {}", sign, coef, cur, next);
            //             return Ok(sign * coef * cur + next);
            //         }
            //     }
            // };

            if exclude_row.is_some() && exclude_row.unwrap() >= matrix.len() {
                Ok(0)
            } else if exclude_row.is_some() {
                let stripped_matrix = subtract_row_and_col(matrix, exclude_row.unwrap(), 0);
                println!("---------------------------------------------");
                let sign = ((exclude_row.unwrap() as i64 % 2) * -2) + 1;
                let coef = matrix[exclude_row.unwrap()][0];
                let cur =
                    calculate_determinant(&stripped_matrix, Some(0), _exclude_col).expect("msg");
                let next =
                    calculate_determinant(matrix, Some(exclude_row.unwrap() + 1), _exclude_col)
                        .expect("msg");
                println!("{} * {} * {} + {}", sign, coef, cur, next);
                Ok(sign * coef * cur + next)
            } else {
                calculate_determinant(matrix, Some(0), Some(0))
            }
        } else {
            println!("---------------------------------------------");
            let result = matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
            println!(
                "Calculating: \n{:?}\n{:?}\nResult: {result}",
                matrix[0], matrix[1]
            );
            Ok(result)
        }
    }

    pub fn subtract_row_and_col(matrix: &Vec<Vec<i64>>, row: usize, col: usize) -> Vec<Vec<i64>> {
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

        let sv1 = matrix::subtract_row_and_col(&v1, 0, 0);
        let sv2 = matrix::subtract_row_and_col(&v1, 0, 1);
        let sv3 = matrix::subtract_row_and_col(&v1, 1, 0);
        let sv4 = matrix::subtract_row_and_col(&v1, 1, 1);

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
