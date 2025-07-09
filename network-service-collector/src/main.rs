use network_service_shared::{CollectorCommandV1, DATA_COLLECTOR_ADDRESS, encode_v1};
use std::collections::VecDeque;
use std::io::Write;
use std::{sync::mpsc::Sender, time::Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("unable to connect to server")]
    UnableToConnect,
    #[error("sending the data failed")]
    UnableToSend,
}

fn main() {
    let uuid = get_uuid();

    let (tx, rx) = std::sync::mpsc::channel::<CollectorCommandV1>();
    // Start the collector thread
    let _collector_thread = std::thread::spawn(move || {
        collect_data(tx, uuid);
    });

    let mut data_queue = VecDeque::with_capacity(120);

    // Listen for commands to send
    while let Ok(command) = rx.recv() {
        let encoded = encode_v1(&command);
        data_queue.push_back(encoded);
        // try and send all queue commands
        let send_result = send_queue(&mut data_queue);
        if let Err(e) = send_result {
            println!("Error sending queue: {e:?}");
        }
    }
}

pub fn collect_data(tx: Sender<CollectorCommandV1>, collector_id: u128) {
    // initialize the sys info data
    let mut sys = sysinfo::System::new_all();

    // perform  a single refresh  and pause. It collects data via deltas
    // so the first reading will be useless
    sys.refresh_memory();
    sys.refresh_cpu_all();
    std::thread::sleep(std::time::Duration::from_secs_f32(1.0));

    // run forever
    loop {
        // note the time we start this loop
        let now = Instant::now();

        // refresh the data
        sys.refresh_memory();
        sys.refresh_cpu_all();

        // get the new values
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let num_cpus = sys.cpus().len();
        let total_cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>();
        let average_cpu_usage = total_cpu_usage / num_cpus as f32;

        // submit the data
        let send_result = tx.send(CollectorCommandV1::SubmitData {
            collector_id: collector_id,
            total_memory: total_memory,
            used_memory: used_memory,
            average_cpu_usage: average_cpu_usage,
        });

        if let Err(e) = send_result {
            println!("Error sending data: {e:?}");
        }

        // wait for next cycle
        let elasped_seconds = now.elapsed().as_secs_f32();
        if elasped_seconds < 1.0 {
            std::thread::sleep(std::time::Duration::from_secs_f32(1.0 - elasped_seconds));
        } else {
            // warning - we are running behind
            std::thread::sleep(std::time::Duration::from_secs_f32(1.0));
        }
    }
}

pub fn send_command(bytes: &[u8]) -> Result<(), CollectorError> {
    println!("sending {} bytes", bytes.len());
    let mut stream = std::net::TcpStream::connect(DATA_COLLECTOR_ADDRESS)
        .map_err(|_| CollectorError::UnableToConnect)?;
    stream
        .write_all(bytes)
        .map_err(|_| CollectorError::UnableToSend)?;

    Ok(())
}

pub fn send_queue(queue: &mut VecDeque<Vec<u8>>) -> Result<(), CollectorError> {
    let mut stream = std::net::TcpStream::connect(DATA_COLLECTOR_ADDRESS)
        .map_err(|_| CollectorError::UnableToConnect)?;

    while let Some(command) = queue.pop_front() {
        println!("sending {} bytes", command.len());
        if stream.write_all(&command).is_err() {
            queue.push_front(command);
            return Err(CollectorError::UnableToSend);
        }
    }

    Ok(())
}

fn get_uuid() -> u128 {
    let path = std::path::Path::new("uuid");
    if path.exists() {
        let contents = std::fs::read_to_string(path).unwrap();
        return contents.parse::<u128>().unwrap();
    } else {
        let uuid = uuid::Uuid::new_v4().as_u128();
        std::fs::write(path, uuid.to_string()).unwrap();
        return uuid;
    }
}
