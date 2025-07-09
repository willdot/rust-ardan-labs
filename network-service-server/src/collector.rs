use network_service_shared::{CollectorCommandV1, DATA_COLLECTOR_ADDRESS, decode_v1};
use sqlx::Pool;
use sqlx::Sqlite;
use std::net::SocketAddr;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

pub async fn data_collector(cnn: Pool<Sqlite>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(DATA_COLLECTOR_ADDRESS).await?;

    loop {
        let (socket, address) = listener.accept().await?;
        tokio::spawn(new_connection(socket, address, cnn.clone()));
    }
}

async fn new_connection(mut socket: TcpStream, address: SocketAddr, cnn: Pool<Sqlite>) {
    println!("new connection from {address:?}");
    let mut buf = vec![0u8; 1024];
    loop {
        let n = socket
            .read(&mut buf)
            .await
            .expect("failed to read data from socket");
        if n == 0 {
            println!("no data received - connection closed");
            return;
        }

        println!("received {n} bytes");
        let received_data = decode_v1(&buf[0..n]);
        println!("received data: {received_data:?}");

        match received_data {
            (
                timestamp,
                CollectorCommandV1::SubmitData {
                    collector_id,
                    total_memory,
                    used_memory,
                    average_cpu_usage,
                },
            ) => {
                let collector_id = uuid::Uuid::from_u128(collector_id);
                let collector_id = collector_id.to_string();
                let result = sqlx::query("INSERT INTO timeseries (collector_id, received, total_memory, used_memory, average_cpu) VALUES ($1, $2, $3, $4, $5)")
                            .bind(collector_id)
                            .bind(timestamp)
                            .bind(total_memory as i64)
                            .bind(used_memory as i64)
                            .bind(average_cpu_usage)
                            .execute(&cnn)
                            .await;

                if result.is_err() {
                    println!("Error inserting data into the database: {result:?}");
                }
            }
        }
    }
}
