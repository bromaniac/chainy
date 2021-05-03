use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};
use sha1::{Digest, Sha1};

#[derive(Serialize, Deserialize, Debug)]
pub struct Chainy {
    chain: Vec<Block>,
}

impl Chainy {
    pub fn new() -> Chainy {
        let genesis = Block::new(
            0,
            "GENESIS".to_owned(),
            "ce02dec31ca49f3c8f149b3b931a0155121d2ca0".to_owned(), //sha1 of GENESIS
        );

        Chainy {
            chain: vec![genesis],
        }
    }

    pub fn entry(&self, data: String) {
        todo!()
    }

    fn add_block(&mut self, b: Block) {
        self.chain.push(b);
    }

    fn validate(&self) -> bool {
        todo!()
    }

    pub fn store(&self, path: &str) {
        fs::write(path, format!("{}", self)).unwrap();
    }

    pub fn load(path: &str) -> Chainy {
        let serialized = fs::read(path).unwrap();
        let deserialized: Chainy = serde_json::from_str(str::from_utf8(&serialized).unwrap()).unwrap();
        
        match deserialized.validate() {
            true => deserialized,
            false => panic!("chain is not valid"),
        }
    }
}

impl fmt::Display for Chainy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let serialized = serde_json::to_string(&self).unwrap();
        write!(f, "{}", serialized)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {
    offset: u64,
    data: String,
    timestamp: u64,
    hash: String,
    previous_hash: String,
}

impl Block {
    fn new(offset: u64, data: String, previous_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get unix epoch time")
            .as_secs();

        let hash = calculate_hash(&offset, &data, timestamp, &previous_hash);

        Block {
            offset,
            data,
            timestamp,
            hash,
            previous_hash,
        }
    }

    fn validate(&self) -> bool {
        let hash = calculate_hash(
            &self.offset,
            &self.data,
            self.timestamp,
            &self.previous_hash,
        );
        match hash == self.hash {
            true => true,
            false => panic!("block is not valid"),
        }
    }
}

fn calculate_hash(offset: &u64, data: &str, timestamp: u64, previous_hash: &str) -> String {
    let mut hasher = Sha1::new();

    let o = offset.to_string();
    let t = timestamp.to_string();

    hasher.update(o + data + &t + previous_hash);

    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn init() {
        let c = crate::Chainy::new();
        print!("{}", c);
    }
}
