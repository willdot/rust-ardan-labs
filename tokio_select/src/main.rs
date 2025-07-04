use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

async fn do_work() {
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("hello do work");
}

async fn timeout(seconds: f32) {
    tokio::time::sleep(Duration::from_secs_f32(seconds)).await;
    println!("hello timeout");
}

async fn receiver(mut rx: mpsc::Receiver<u32>, mut broadcast_rx: broadcast::Receiver<u32>) {
    loop {
        tokio::select! {
            Some(n) = rx.recv() => println!("Received message {n} on the mpsc channel"),
            Ok(n) = broadcast_rx.recv() => println!("Received message {n} on the broadcast channel"),
        }
    }
}

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = do_work() => println!("do work finished first"),
        _ = timeout(1.0) => println!("timeout finished first"),
    }

    // note: when the select has finished and one of the functions being waited on has finished, the other one is cancelled and cleared up.
    // that means that the print statement of the function that didn't finish first will not be printed.
    tokio::time::sleep(Duration::from_secs(5)).await;

    let (tx, rx) = mpsc::channel::<u32>(1);
    let (broadcast_tx, broadcast_rx) = broadcast::channel::<u32>(1);

    tokio::spawn(receiver(rx, broadcast_rx));

    for count in 0..10 {
        if count % 2 == 0 {
            tx.send(count).await.unwrap();
        } else {
            broadcast_tx.send(count).unwrap();
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
