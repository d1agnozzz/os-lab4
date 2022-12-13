mod lib;
use lib::matrix::subtract_row_and_col;
fn main() {
    println!("Hello, world!");
    let v = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

    let sv = subtract_row_and_col(&v, 0, 0);

    println!("{:?}", &v);
    println!("{:?}", &sv);
}

use std::sync::mpsc::channel;
use std::thread;

fn channels() {
    // let (tx, rx) = mpsc
}


