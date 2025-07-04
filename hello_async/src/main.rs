use futures::executor::block_on;
use futures::future::join_all;
use futures::join;

fn main() {
    block_on(say_hello());

    println!("something");
}

fn do_something_sync() {
    println!("not async!");
}

async fn say_hello() {
    println!("hello");
    second_function().await;
    join!(second_function(), say_goodbye());

    let n: u32 = double(4).await;
    println!("{n}");

    let futures = vec![double(1), double(2)];
    let results = join_all(futures).await;
    println!("{results:?}");

    do_something_sync();
}

async fn second_function() {
    println!("hello again");
}

async fn say_goodbye() {
    println!("goodbye");
}

async fn double(n: u32) -> u32 {
    return n * 2;
}
