// This is how to do it manually
// fn main() {
//     let rt = runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()
//         .unwrap();

//     rt.block_on(hello());
// }

use std::time::Duration;
use tokio::task::spawn_blocking;

// This is how to do it so that it's all done for you
#[tokio::main]
async fn main() {
    hello().await;

    tokio::join!(hello(), goodbye());

    let result = tokio::join!(hello(), goodbye());
    println!("{result:?}");

    // even though ticker is called in the background and then we are waiting for hello to finish, there
    // is no guarantee that ticker will finish or run before hello finishes.
    // tokio::spawn(ticker());
    // hello().await;
    println!("--------");

    // this way is better so we know that ticker will finish
    let _ = tokio::join!(
        tokio::spawn(ticker()),
        tokio::spawn(hello()),
        tokio::spawn(ticker())
    );

    println!("finished");

    tokio::join!(
        hello_delay_blocking(1, 500),
        hello_delay_blocking(2, 1000),
        hello_delay_blocking(3, 1500),
    );

    tokio::join!(
        hello_delay_non_block(1, 500),
        hello_delay_non_block(2, 1000),
        hello_delay_non_block(3, 1500),
    );
}

async fn hello() -> u32 {
    println!("hello tokio");
    return 1;
}

async fn goodbye() -> u32 {
    println!("goodbye tokio");
    return 2;
}

async fn ticker() {
    for i in 0..10 {
        println!("tick {i}");
        // this allows other async functions to run their work before the next iteration of this loop runs
        tokio::task::yield_now().await;
    }
}

async fn hello_delay_blocking(task: u64, time: u64) {
    println!("task {task} has started (blocking)");
    // using this type of sleep from a task inside a tokio thread it will make the tokio thread sleep so no other tokio tasks can run
    std::thread::sleep(Duration::from_millis(time));
    println!("task {task} has finished (blocking)");
}

async fn hello_delay_non_block(task: u64, time: u64) {
    println!("task {task} has started (non blocking)");
    // doing it like this will just send the task in this function to sleep not the entire thread
    tokio::time::sleep(Duration::from_millis(time)).await;
    println!("task {task} has finished (non blocking)");
}
