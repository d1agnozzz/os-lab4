use ipc_channel::ipc::TryRecvError;
use lab4::matrix::{
    self, calculate_determinant, interprocess_determinant_calculation, print_matrix,
};
use nix::sys::wait::WaitStatus;
use nix::unistd::{fork, ForkResult};


fn main() {
    // let (tx, rx) = ipc_channel::ipc::channel().unwrap();

    // match unsafe { fork() } {
    //     Ok(ForkResult::Parent { child }) => {
    //         std::thread::sleep(std::time::Duration::from_millis(100));
    //         println!("I'm parent");
    //         drop(tx);
    //         loop {
    //             match rx.try_recv() {
    //                 Ok(res) => {
    //                     // Do something interesting with your result
    //                     println!("Received data... {res}");
    //                 }
    //                 Err(TryRecvError::Empty) => {
    //                     // Do something else useful while we wait
    //                     println!("Emptied the receiver...");
    //                     std::thread::sleep(std::time::Duration::from_millis(100));
    //                 }
    //                 Err(TryRecvError::IpcError(err)) => match err {
    //                     ipc_channel::ipc::IpcError::Bincode(_) => println!("Bincode error"),
    //                     ipc_channel::ipc::IpcError::Io(_) => println!("IO error"),
    //                     ipc_channel::ipc::IpcError::Disconnected => {
    //                         println!("Receiver disconnected");
    //                         break;
    //                     }
    //                 },
    //             }
    //         }
    //         println!("Finished parent process.");
    //     }
    //     Ok(ForkResult::Child) => {
    //         for i in 0..100 {
    //             tx.send(format!("{i} message from child")).unwrap();
    //             std::thread::sleep(std::time::Duration::from_millis(10));
    //         }
    //     }
    //     Err(_) => println!("Fork failed!"),
    // }
    // Матрица
    let matrix_1 = vec![
        vec![4, 7, 9, 5],
        vec![3, 6, 9, 4],
        vec![0, 4, 26, 8],
        vec![9, 6, 3, 7],
    ];
    let matrix_2 = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

    let matrix_3 = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
        vec![13, 14, 15, 16],
    ];

    const VERBOSE: bool = false;

    ipc_channels(&matrix_1, VERBOSE);
    ipc_channels(&matrix_2, VERBOSE);
    ipc_channels(&matrix_3, VERBOSE);
}

use std::os::unix::net::SocketAddr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

fn channels_archieved() {
    let (txn, rx) = channel();
    let v5 = Arc::new(vec![
        vec![43, 76, -65, -12],
        vec![22, -87, 99, -22],
        vec![-76, 89, 57, -43],
        vec![43, -75, -62, 18],
    ]);
    let mut handles = vec![];

    for t in 0..v5.len() {
        let v5c = v5.clone();

        let txi = txn.clone();

        let handle = thread::spawn(move || {
            let mut minor = matrix::select_minor(&v5c, t, 0);

            for r in 1..=v5c.len() - 2 {
                for c in 1..=v5c.len() - 2 {
                    minor = matrix::select_minor(&minor, r, c);

                    if minor.len() == 2 {
                        let coeff = (((r + c) % 2) as i64 * -2) + 1;
                        let det = calculate_determinant(&minor, None, None);
                        txi.send(coeff * det.unwrap());
                    }
                }
            }
        });
        handles.push(handle);
    }

    let mut total_det = 0;
    if let Ok(r) = rx.recv() {
        total_det += r;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("{}", total_det);
}

fn ipc_channels(matrix: &Vec<Vec<i64>>, verbose: bool) {
    let (tx, rx) = ipc_channel::ipc::channel().expect("Could not create origin channel");

    interprocess_determinant_calculation(matrix, tx.clone(), 1, true, verbose);
    drop(tx);

    let mut determinant = 0;
    loop {
        match rx.try_recv() {
            Ok(res) => {
                // Do something interesting with your result
                if verbose {
                    println!("Received data at origin channel {res}");
                }
                determinant += res;
            }
            Err(TryRecvError::Empty) => {
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
    println!("Finished calculating! Determinant is {determinant}");
    println!("Matrix is:");
    print_matrix(&matrix);
}
