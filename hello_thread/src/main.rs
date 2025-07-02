use dashmap::DashMap;
use std::{
    io::{self, Read},
    mem,
    sync::{
        LazyLock, Mutex, RwLock,
        atomic::{AtomicBool, AtomicI32},
    },
    thread::{self, Scope, Thread},
    time::Duration,
};
const N_THREADS: usize = 8;

fn hello_thread(n: u32) {
    println!("hello from thread {n}");
}

fn do_math(i: u32) -> u32 {
    println!("hello from do math thread {i} ");
    let mut n = i + 1;

    for _ in 0..10 {
        n *= 2;
    }
    return n;
}

fn main() {
    println!("Hello from the main thread");

    println!("running hello threads");
    run_hello_threads();

    println!("running do math threads");
    run_do_math_threads();

    println!("running divide work up threads");
    run_divide_work();

    println!("running thread builder");
    run_thread_builder();

    println!("running scoped thread");
    run_scoped_threads();

    println!("running footgun... never do this in production!");
    run_footgun();

    println!("running atomics");
    run_atomics();

    println!("running mutexs");
    run_mutexes();

    println!("running read/write lock");
    run_read_write_lock();

    println!("running deadlocks");
    run_dead_locks();

    println!("running poisioner");
    run_poisioner();

    println!("runnning lock free");
    run_lock_free();

    println!("running parking thread");
    run_parked_thread();
}

fn run_hello_threads() {
    let mut thread_handles = Vec::new();
    for i in 0..5 {
        let thread_handle = thread::spawn(move || hello_thread(i));
        thread_handles.push(thread_handle);
    }

    thread_handles.into_iter().for_each(|h| h.join().unwrap());
}

fn run_do_math_threads() {
    let mut thread_handles = Vec::new();
    for i in 0..10 {
        let thread_handle = thread::spawn(move || do_math(i));
        thread_handles.push(thread_handle);
    }

    thread_handles.into_iter().for_each(|h| {
        println!("{}", h.join().unwrap());
    });
}

fn run_divide_work() {
    let to_add: Vec<u32> = (0..5000).collect();
    let mut thread_handles = Vec::new();
    let chunks = to_add.chunks(N_THREADS);

    for chunk in chunks {
        let my_chunk = chunk.to_owned();
        thread_handles.push(thread::spawn(move || {
            return my_chunk.iter().sum::<u32>();
        }));
    }

    // total of each chunks sum
    let mut sum = 0;
    for handle in thread_handles {
        sum += handle.join().unwrap();
    }

    println!("sum is {sum}")
}

fn run_thread_builder() {
    thread::Builder::new()
        .name("Named Thread".to_string())
        .stack_size(std::mem::size_of::<usize>() * 4)
        .spawn(my_thread)
        .unwrap()
        .join()
        .unwrap();
}

fn my_thread() {
    println!(
        "hello from a thread named '{}'",
        thread::current().name().unwrap()
    );
}

fn run_scoped_threads() {
    let to_add: Vec<u32> = (0..5000).collect();
    let chunks = to_add.chunks(N_THREADS);

    let sum = thread::scope(|s| {
        let mut thread_handles = Vec::new();

        for chunk in chunks {
            let thread_handle = s.spawn(move || {
                return chunk.iter().sum::<u32>();
            });
            thread_handles.push(thread_handle);
        }

        thread_handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .sum::<u32>()
    });

    println!("sum: {sum}");
}

static mut FOOTGUN_COUNTER: i32 = 0;

fn run_footgun() {
    let mut handles = Vec::new();
    for _ in 0..1_000 {
        let handle = std::thread::spawn(|| {
            for _ in 0..1_1000 {
                // LOL don't do this
                unsafe {
                    FOOTGUN_COUNTER += 1;
                }
            }
        });
        handles.push(handle);
    }
    handles.into_iter().for_each(|h| h.join().unwrap());

    unsafe {
        let result = FOOTGUN_COUNTER;
        println!("result is {result}");
    }
}

static ATOMIC_COUNTER: AtomicI32 = AtomicI32::new(0);

fn run_atomics() {
    let mut handles = Vec::new();
    for _ in 0..1_000 {
        let handle = std::thread::spawn(|| {
            for _ in 0..1_1000 {
                ATOMIC_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    handles.into_iter().for_each(|h| h.join().unwrap());

    println!(
        "result is {}",
        ATOMIC_COUNTER.load(std::sync::atomic::Ordering::Relaxed)
    )
}

static NUMBERS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn run_mutexes() {
    let mut handles = Vec::new();
    for _ in 0..10 {
        let handle = thread::spawn(|| {
            let mut lock = NUMBERS.lock().unwrap();
            lock.push(1);
        });
        handles.push(handle);
    }

    handles.into_iter().for_each(|h| h.join().unwrap());

    let lock = NUMBERS.lock().unwrap();
    println!("{:#?}", lock);
}

static USERS: LazyLock<RwLock<Vec<String>>> = LazyLock::new(|| RwLock::new(build_users()));
static ATOMIC_QUIT: AtomicBool = AtomicBool::new(false);

fn build_users() -> Vec<String> {
    vec!["Alice".to_string(), "Bob".to_string()]
}

fn read_line() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    return input.trim().to_string();
}

fn run_read_write_lock() {
    thread::spawn(|| {
        loop {
            if ATOMIC_QUIT.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            println!("current users (in a thread)");
            let users = USERS.read().unwrap();
            println!("{users:?}");
            thread::sleep(std::time::Duration::from_secs(3));
        }
    });

    loop {
        println!("enter a name to add to the user list (or q to quite)");
        let input = read_line();

        if input == "q" {
            break;
        }

        let mut lock = USERS.write().unwrap();
        lock.push(input);
    }
    ATOMIC_QUIT.store(true, std::sync::atomic::Ordering::Relaxed);
}

static MY_SHARED: Mutex<i32> = Mutex::new(3);

fn run_dead_locks() {
    let lock = MY_SHARED.lock().unwrap();
    std::mem::drop(lock);

    if let Ok(_lock) = MY_SHARED.try_lock() {
        println!("got the lock");
        return;
    }
    println!("haven't got the lock");
}

fn run_poisioner() {
    let handle = thread::spawn(poisoner);
    println!("trying to return from the thread");
    println!("{:?}", handle.join());

    let lock = MY_SHARED.lock();
    println!("{lock:?}");

    let recovered_data = lock.unwrap_or_else(|poisoned| {
        println!("mutex was poisioned, recovering data");
        poisoned.into_inner()
    });

    println!("{}", recovered_data)
}

fn poisoner() {
    let mut lock = MY_SHARED.lock().unwrap();
    *lock += 2;
    panic!("the poisoner strikes");
}

static SHARED: LazyLock<DashMap<u32, u32>> = LazyLock::new(DashMap::new);

fn run_lock_free() {
    for i in 0..100 {
        thread::spawn(move || {
            loop {
                if let Some(mut v) = SHARED.get_mut(&i) {
                    *v += 1;
                } else {
                    SHARED.insert(i, i);
                }
            }
        });
    }

    thread::sleep(Duration::from_secs(2));
    println!("{SHARED:#?}");
}

fn run_parked_thread() {
    let mut threads = Vec::new();
    for i in 0..10 {
        let thread = thread::spawn(move || {
            parkable_thread(i);
        });
        threads.push(thread);
    }

    loop {
        println!("thread to unpark:");
        let input = read_line();

        if input == "q" {
            break;
        }

        if let Ok(number) = input.parse::<usize>() {
            if number < 10 {
                threads[number].thread().unpark();
            }
        }
    }
}

fn parkable_thread(n: u32) {
    loop {
        thread::park();
        println!("thread {n} is unparked briefly");
    }
}
