use serde_derive::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use core::str;
use std::fmt::Write;
use rocksdb::DB;

use chrono::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f32,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Blockheader {
    timestamp: i64,
    nonce: u32,
    pre_hash: String,
    merkle: String,
    difficulty: u32,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Block {
    header: Blockheader,
    count: u32,
    transactions: Vec<Transaction>,
}

pub struct Chain {
    db: DB,
    height: u32,
    curr_trans: Vec<Transaction>,
    difficulty: u32,
    miner_addr: String,
    reward: f32,
}

// can only create one instances of the struct
impl Chain {
    pub fn new(miner_addr: String, difficulty: u32, db: DB) -> Chain {
        let mut chain = Chain {
            db,
            height: 0,
            curr_trans: Vec::new(),
            difficulty,
            miner_addr,
            reward: 100.0,
        };
        let height = chain.get_height();
        if height == 0 {
            chain.generate_new_block();
        }else {
            chain.height = height;
        }
        chain
    }

    pub fn get_height(&self) -> u32 {
        match self.db.get("height") { // do not convert to bytes
            Ok(Some(height)) => {
                let digits: [u8; 4] = height.try_into().unwrap();
                u32::from_be_bytes(digits)
            },
            _ => 0
        }
    }

    pub fn reveal_chain(&self) {
        for i in 0..self.height {
            let block = self.get_block_by_index(i).unwrap();
            println!("Block hash: {}",Chain::hash(&block.header));
            println!("{:#?}", block)
        }
    }

    pub fn new_transaction(&mut self, sender: String, receiver: String, amount: f32) -> bool {
        self.curr_trans.push(Transaction {
            sender,
            receiver,
            amount,
        });

        true
    }

    pub fn last_hash(&self) -> Result<String, &str> {
        if self.height == 0 {
            return Ok(String::from_utf8(vec![48; 64]).unwrap());
        }
        if let Ok(hash) = self.get_hash_by_index(self.height - 1) {
            Ok(hash)
        } else {
            Err("Hash not found for current index")
        }
    }

    pub fn get_block_by_index(&self, index: u32) -> Result<Block, &str> {
        if let Ok(hash) = self.get_hash_by_index(index) {
            match self.get_block_by_hash(hash) {
                Ok(block) => Ok(block),
                Err(_) => Err("Block not found"),
            }
        } else {
            Err("Block not found")
        }
    }

    pub fn get_hash_by_index(&self, index: u32) -> Result<String, &str>{
        match self.db.get(index.to_be_bytes()) {
            Ok(Some(hash)) => {
                Ok(String::from_utf8(hash).unwrap())
            },
            Ok(None) => {
                Err("Block not found")
            },
            Err(_) => Err("operational error occured")
        }
    }

    pub fn get_block_by_hash(&self, hash: String) -> Result<Block, &'static str> {
        match self.db.get(hash.as_bytes()) {
            Ok(Some(block)) => {
                Ok(serde_json::from_str(str::from_utf8(&block).unwrap()).unwrap())
            },
            Ok(None) => {
                Err("Block not found")
            },
            Err(_) => Err("operational error occured")
        }
    }

    pub fn update_difficulty(&mut self, difficulty: u32) -> bool {
        self.difficulty = difficulty;
        true
    }

    pub fn update_reward(&mut self, reward: f32) -> bool {
        self.reward = reward;
        true
    }

    pub fn generate_new_block(&mut self) -> bool {
        if !(self.height == 0) && self.curr_trans.is_empty() {
            println!("No transaction to add!!");
            return false;
        }

        let header = Blockheader {
            timestamp: Utc::now().timestamp_millis(),
            nonce: 0,
            pre_hash: self.last_hash().unwrap(),
            merkle: String::new(),
            difficulty: self.difficulty,
        };

        let reward_trans = Transaction {
            sender: String::from("Root"),
            receiver: self.miner_addr.clone(),
            amount: self.reward,
        };

        let mut block = Block {
            header: header.clone(),
            count: 0,
            transactions: vec![],
        };

        block.transactions.push(reward_trans);
        block.transactions.append(&mut self.curr_trans);
        block.count = block.transactions.len() as u32;
        block.header.merkle = Chain::get_merkle(block.transactions.clone());
        Chain::proof_of_work(&mut block.header);

        println!("{:#?}", &block);

        let block_hash = Chain::hash(&block.header);

        if self.db.put(block_hash.as_bytes(), serde_json::to_string(&block).unwrap().as_bytes()).is_err() {
            return false;
        }

        if self.db.put(self.height.to_be_bytes(), block_hash.as_bytes()).is_err() {
            return false;
        }
        
        if self.db.put("height", (self.height + 1).to_be_bytes()).is_err() {
            return false;
        }
        self.height += 1;
        self.db.flush().expect("Failed to add the data to the db");
        true
    }

    fn get_merkle(curr_trans: Vec<Transaction>) -> String {
        let mut merkle = Vec::new();

        for t in &curr_trans {
            let hash = Chain::hash(t);
            merkle.push(hash);
        }

        if merkle.len() % 2 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last);
        }

        while merkle.len() > 1 {
            let mut h1 = merkle.remove(0);
            let mut h2 = merkle.remove(0);
            h1.push_str(&mut h2);
            let nh = Chain::hash(&h1);
            merkle.push(nh);
        }
        merkle.pop().unwrap()
    }

    pub fn proof_of_work(header: &mut Blockheader) {
        loop {
            let hash = Chain::hash(header);
            let slice = &hash[..header.difficulty as usize];
            match slice.parse::<u32>() {
                Ok(val) => {
                    if val != 0 {
                        header.nonce += 1;
                    } else {
                        println!("Block hash: {}", hash);
                        break;
                    }
                }
                Err(_) => {
                    header.nonce += 1;
                    continue;
                }
            };
        }
    }

    pub fn hash<T: serde::Serialize>(item: &T) -> String {
        let input = serde_json::to_string(&item).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let res = hasher.finalize();
        let vec_res = res.to_vec();

        Chain::hex_to_string(vec_res.as_slice())
    }

    pub fn hex_to_string(vec_res: &[u8]) -> String {
        let mut s = String::new();
        for b in vec_res {
            write!(&mut s, "{:x}", b).expect("unable to write");
        }
        s
    }
}