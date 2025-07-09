use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DATA_COLLECTOR_ADDRESS: &str = "127.0.0.1:9004";
const MAGIC_NUMBER: u16 = 1234;
const VERSION_NUMBER: u16 = 1;

fn unix_now() -> u32 {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backward");
    return since_epoch.as_secs() as u32;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CollectorCommandV1 {
    SubmitData {
        collector_id: u128,
        total_memory: u64,
        used_memory: u64,
        average_cpu_usage: f32,
    },
}

pub fn encode_v1(command: &CollectorCommandV1) -> Vec<u8> {
    // let json = serde_json::to_string(&command).unwrap();
    // let json_bytes = json.as_bytes();
    let payload_bytes = bincode::serialize(command).unwrap();
    let crc = crc32fast::hash(&payload_bytes);
    let payload_size = payload_bytes.len() as u32;
    let timestamp = unix_now();

    let mut result = Vec::with_capacity(140);
    result.extend_from_slice(&MAGIC_NUMBER.to_be_bytes());
    result.extend_from_slice(&VERSION_NUMBER.to_be_bytes());
    result.extend_from_slice(&timestamp.to_be_bytes());
    result.extend_from_slice(&payload_size.to_be_bytes());
    result.extend_from_slice(&payload_bytes);
    result.extend_from_slice(&crc.to_be_bytes());

    return result;
}

pub fn decode_v1(bytes: &[u8]) -> (u32, CollectorCommandV1) {
    let magic_number = u16::from_be_bytes([bytes[0], bytes[1]]);
    let version_number = u16::from_be_bytes([bytes[2], bytes[3]]);
    let timestamp = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let payload_size = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let payload = &bytes[12..12 + payload_size as usize];
    let crc = u32::from_be_bytes([
        bytes[12 + payload_size as usize],
        bytes[13 + payload_size as usize],
        bytes[14 + payload_size as usize],
        bytes[15 + payload_size as usize],
    ]);

    // verify magic number
    assert_eq!(magic_number, MAGIC_NUMBER);

    // verify version number
    assert_eq!(version_number, VERSION_NUMBER);

    // verify crc
    let computed_crc = crc32fast::hash(payload);
    assert_eq!(crc, computed_crc);

    (timestamp, bincode::deserialize(payload).unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{CollectorCommandV1, decode_v1, encode_v1};

    use super::*;

    #[test]
    fn test_encode_decode() {
        let command = CollectorCommandV1::SubmitData {
            collector_id: 1234,
            total_memory: 100,
            used_memory: 50,
            average_cpu_usage: 0.5,
        };

        let encoded = encode_v1(&command);

        let (timestamp, decoded) = decode_v1(&encoded);

        assert_eq!(decoded, command);
        assert!(timestamp > 0);
    }
}
