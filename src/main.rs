use ipc_channel::ipc::TryRecvError;
use lab4::matrix::*;
use nix::sys::wait::WaitStatus;
use nix::unistd::{fork, ForkResult};



fn main() {
    // Матрица
    let matrix_1 = vec![
        vec![4, 7, 9, 5],
        vec![3, 6, 9, 4],
        vec![0, 4, 26, 8],
        vec![9, 6, 3, 7],
    ];
    let matrix_2 = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

    // let raw = matrix_2.as_ptr();

    // let r = v.as_ptr();
    let shmem = match shared_memory::ShmemConf::new().size(8).create() {
        Ok(m) => m,
        Err(_) => return,
    };

    // let raw_shm = shmem.as_ptr();

    // let bytes: usize = 0x123456789abcdef0;
    // println!("original bytes: {:x}", bytes);
    // for i in 1..=7_usize {
    //     let shift: usize = 256;
    //     let byte: u8 = (bytes / (shift.pow(8 - i as u32))) as u8;
    //     println!("{:x}", byte);
    // }

    // unsafe {
    //     println!("first byte in shm before: {:x}", *raw_shm);
    //     write_usize_to_shm(raw_shm, bytes as usize);
    //     println!("1st byte in shm after: {:x}", *raw_shm);
    //     println!("2nd byte in shm after: {:x}", *raw_shm.add(1));
    //     println!("7th byte in shm after: {:x}", *raw_shm.add(6));
    //     println!("8th byte in shm after: {:x}", *raw_shm.add(7));

    //     let readed = read_usize_from_shm(raw_shm);
    //     println!("readed value: {:x}", readed);
    // }

    

    // unsafe {
    //     let mut v = Vec::new();

    //     let a = raw as usize;
    //     let b = a as *const Vec<i64>;


    //     for i in 0..(*raw).len() {
    //         v.push((*raw.add(i)).clone());
    //     }
    //     println!("sizeof raw {}", std::mem::size_of::<*const Vec<i64>>());
    //     print_matrix(&v);
    // }

    let matrix_3 = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
        vec![13, 14, 15, 16],
    ];

    const VERBOSE: bool = false;

    ipc_channels(&matrix_3, VERBOSE);
    // ipc_channels(&matrix_2, VERBOSE);
    // ipc_channels(&matrix_3, VERBOSE);



    sh_mem(&matrix_1);

    // match  unsafe{fork()} {
    //     Ok(ForkResult::Parent { child }) => {
    //         while nix::sys::wait::wait().expect("'wait' error ") == WaitStatus::StillAlive {
    //         }
    //         println!("Child process terminated");
    //     },
    //     Ok(ForkResult::Child) => {
    //         for i in 0..100 {
    //             println!("{i}");
    //             std::thread::sleep(std::time::Duration::from_millis(1));
    //         }
    //     },
    //     Err(_) => (),
    // }
    // println!("sizeof {}", std::mem::size_of::<Vec<i8>>());
    // println!("sizeof {}", std::mem::size_of::<usize>());
    // println!("sizeof {}", std::mem::size_of::<i32>());
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
            let mut minor = select_minor(&v5c, t, 0);

            for r in 1..=v5c.len() - 2 {
                for c in 1..=v5c.len() - 2 {
                    minor = select_minor(&minor, r, c);

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

    channel_determinant_calculation(matrix, tx.clone(), 1, true, verbose);
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
            Err(TryRecvError::Empty) => {}
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

fn sh_mem(matrix: &Vec<Vec<i64>>) {

    let shmem = shared_memory::ShmemConf::new().size(8).create().expect("Shmem failed at main");

    
    unsafe {
        // write_usize_to_shm(shmem.as_ptr(), matrix.as_ptr() as usize);
        shm_determinant_calculation(matrix, shmem.as_ptr(), 1, true);
        thread::sleep(std::time::Duration::from_millis(1000));
        let determinant = read_usize_from_shm(shmem.as_ptr());
        println!("Shmem determinant is {}", determinant as i64);
    }


}