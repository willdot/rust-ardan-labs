use once_cell::sync::Lazy;
use std::{collections::VecDeque, sync::Mutex, time::Duration};

static WORK_QUEUE: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));

fn main() {
    let cpu_count = num_cpus::get();

    let mut threads = Vec::with_capacity(cpu_count);
    let mut broadcast = Vec::with_capacity(cpu_count);

    for cpu in 0..cpu_count {
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        broadcast.push(tx);

        let thread = std::thread::spawn(move || {
            while rx.recv().is_ok() {
                let mut lock = WORK_QUEUE.lock().unwrap();
                if let Some(work) = lock.pop_front() {
                    std::mem::drop(lock);
                    println!("CPU {cpu} got work: {work}");
                    std::thread::sleep(Duration::from_secs(2));
                    println!("CPU {cpu} finished!");
                } else {
                    println!("CPU {cpu} found no work");
                }
            }
        });
        threads.push(thread);
    }

    loop {
        let sent = {
            let mut lock = WORK_QUEUE.lock().unwrap();
            let len = lock.len();
            println!("There are {len} items in the queue");
            if len < 5 {
                lock.push_back("Hello".to_string());
                true
            } else {
                false
            }
        };
        if sent {
            broadcast.iter().for_each(|tx| tx.send(()).unwrap());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
