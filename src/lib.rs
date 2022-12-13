pub mod matrix {
    #[derive(Debug)]
    pub enum MatrixCalcError {
        NonSquare,
    }

    #[derive(Clone, Copy)]
    pub struct Pivot {
        row: usize,
        col: usize,
        val: i32,
    }

    impl Pivot {
        fn offset(self, x: usize, y: usize, new_val: i32) -> Pivot {
            Pivot {
                row: x,
                col: y,
                val: new_val,
            }
        }
    }

    pub fn calculate_determinant(
        matrix: &Vec<Vec<i32>>,
        pivot: &Pivot,
    ) -> Result<i32, MatrixCalcError> {
        if matrix.len() != matrix[0].len() {
            return Err(MatrixCalcError::NonSquare);
        }

        let mut determinant = 0;

        if matrix.len() > 2 {
            for r in 0..matrix.len() {
                let stripped_matrix = subtract_row_and_col(matrix, r, 0);
                determinant +=
                    calculate_determinant(&stripped_matrix, &pivot.offset(r, 0, matrix[r][0]))
                        .expect("msg");
            }
        }

        if matrix.len() == 2 {
            
        }

        todo!()
    }

    pub fn subtract_row_and_col(matrix: &Vec<Vec<i32>>, row: usize, col: usize) -> Vec<Vec<i32>> {
        let mut new_matrix: Vec<Vec<i32>> = Vec::new();

        for r in 0..matrix.len() {
            if r == row {
                continue;
            }
            let mut current_row: Vec<i32> = Vec::new();
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
}
