// MIT License
//
// Copyright (c) 2021 Fredrik Broman
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{convert::TryInto, fmt};
use std::{fs, str};
use thiserror::Error;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Chainy {
    chain: Vec<Block>,
}

impl Chainy {
    pub fn new() -> MyResult<Chainy> {
        let genesis = Block::new(
            0,
            "GENESIS".to_owned(),
            r#"ce02dec31ca49f3c8f149b3b931a0155121d2ca0"#.to_owned(), //sha1 of GENESIS
        )?;

        Ok(Chainy {
            chain: vec![genesis],
        })
    }

    pub fn entry(&mut self, data: &str) -> MyResult<()> {
        if data.len() > 64 {
            return Err(Box::new(ChainyError::DataTooLong));
        }

        let offset = (self.chain.len() + 1).try_into()?;
        let previous_hash = &self.chain.last().ok_or("add block entry failed")?.hash;
        let block = Block::new(offset, data.to_string(), previous_hash.to_string())?;

        self.add_block(block);
        Ok(())
    }

    fn add_block(&mut self, b: Block) {
        self.chain.push(b);
    }

    fn validate(&self) -> MyResult<()> {
        if self.chain[0].offset != 0 {
            return Err(Box::new(ChainyError::ChainNotValid));
        }
        if self.chain[0].previous_hash != r#"ce02dec31ca49f3c8f149b3b931a0155121d2ca0"# {
            return Err(Box::new(ChainyError::ChainNotValid));
        }
        self.chain[0].validate()?;

        for w in self.chain.windows(2) {
            w[1].validate()?;
            if w[0].hash != w[1].previous_hash {
                return Err(Box::new(ChainyError::ChainNotValid));
            }
        }

        Ok(())
    }

    pub fn store(&self, path: &str) -> MyResult<()> {
        fs::write(path, format!("{}", self))?;
        Ok(())
    }

    pub fn load(path: &str) -> MyResult<Chainy> {
        let serialized = fs::read(path)?;
        let deserialized: Chainy = serde_json::from_str(str::from_utf8(&serialized)?)?;

        match deserialized.validate() {
            Ok(_) => Ok(deserialized),
            Err(_) => Err(Box::new(ChainyError::ChainNotValid)),
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
    fn new(offset: u64, data: String, previous_hash: String) -> MyResult<Block> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let hash = calculate_hash(&offset, &data, timestamp, &previous_hash);

        Ok(Block {
            offset,
            data,
            timestamp,
            hash,
            previous_hash,
        })
    }

    fn validate(&self) -> MyResult<()> {
        let hash = calculate_hash(
            &self.offset,
            &self.data,
            self.timestamp,
            &self.previous_hash,
        );
        match hash == self.hash {
            true => Ok(()),
            false => Err(Box::new(ChainyError::BlockNotValid)),
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

#[derive(Error, Debug)]
pub enum ChainyError {
    #[error("block is not valid")]
    BlockNotValid,
    #[error("chain is not valid")]
    ChainNotValid,
    #[error("block data is > 64 chars")]
    DataTooLong,
}

#[cfg(test)]
mod tests {
    #[test]
    fn init() {
        let mut c = crate::Chainy::new().unwrap();
        c.entry("foo").unwrap();
        c.validate().unwrap();
        print!("{}", c);
    }
}
