use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Command {
    pub binary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub invocations: Vec<String>,
    pub guid: u64
}

impl Command {

    pub fn generate_guid() -> u64 {
        use std::fs::OpenOptions;
        use std::io::Read;

        let uri = "/dev/urandom";

        let mut file = OpenOptions::new()
            .read(true)
            .open(uri)
            .expect("Could not open /dev/urandom");

        let mut buffer = [0u8; 8];

        file.read_exact(&mut buffer)
            .expect("Could not read from /dev/urandom");

        u64::from_ne_bytes(buffer)
    }

}


