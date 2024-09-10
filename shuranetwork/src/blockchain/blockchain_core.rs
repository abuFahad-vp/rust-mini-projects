use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use serde_derive::{Serialize, Deserialize};
use rocksdb::Options;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use core::str;
use std::collections::HashMap;
use std::fmt::Write;
use crate::template;
use crate::utils::get_value;
use rocksdb::DB;
use std::process::Command;
use std::fs::File;
use std::io::copy;
use std::fs;
use std::path::Path;
use chrono::prelude::*;
use super::peer_network::{Message, Node};
use super::blockchain_rest::Len;
use super::blockchain_rest::Msg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: f32,
    transaction_id: String,
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
    pub header: Blockheader,
    pub count: u32,
    pub transactions: HashMap<String, Transaction>,
}

pub struct Chain {
    pub db: Option<DB>,
    height: u32,
    curr_trans: HashMap<String, Transaction>,
    difficulty: u32,
    miner_addr: String,
    reward: f32,
    pub uuid: Uuid,
    pub node: Node,
    pub msg_incoming_rx: Receiver<Message>,
    pub msg_outgoing_tx: Sender<Message>
}

// can only create one instances of the struct
impl Chain {
    pub async fn new(port: u16, server_port: u16) -> Chain {

        let node = Node::new(port,format!("127.0.0.1:{server_port}")).await;

        node.server_listen().await;

        // getting the peers
        let peers = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

        Self::get_peers(peers.clone()).await;

        for peer in peers.lock().await.iter() {
            node.add_peer(peer.to_string()).await;
            println!("peers = {}",peer);
        }

        let chain = Self::start_chain(node).await;
        chain
    }

    async fn start_chain(node: Node) -> Chain {
        let mut init_page = template::MenuBuilder::new();
        let node_clone = node.clone();
        init_page.add("1", "Sync chain", move || {
            let node = node_clone.clone();
            async move {
                if node.peer_server_addr.lock().await.len() > 0 {
                    println!("Syncing....");
                    false
                } else {
                    println!("No peer found to sync. Retry");
                    true
                }
            }
        });

        init_page.add("0", "Exit from syncing page", move || {
            async move {
                false
            }
        });

        init_page.run_menu().await;

        // open the db and check the height
        let db_path = "amanah.db";
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        let db = DB::open(&db_opts, db_path).unwrap();

        let mut chain = Chain {
            db: Some(db),
            height: 0,
            curr_trans: HashMap::new(),
            difficulty: 2,
            miner_addr: "".to_string(),
            reward: 100.0,
            uuid: node.get_id(),
            node: node.clone(),
            msg_incoming_rx: node.msg_incoming_rx.clone(),
            msg_outgoing_tx: node.msg_outgoing_tx.clone()
        };

        let height = chain.get_height().await;
        if height <= 0 {
            if node.peer_server_addr.lock().await.len() <= 0 {
                chain.generate_new_block().await;
            } else {
                println!("Syncing the chain....");
                return Self::sync_chain(chain).await;
            }
        } else {
            chain.height = height;
        }
        chain
    }

    async fn get_peers(peers: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>) {

    // adding peer
        let peers_clone = peers.clone();
        let mut peer_page = template::MenuBuilder::new();

        peer_page.add("1", "Add peer", move || {
            let peers_clone = peers_clone.clone();
            async move {
                let mut peers = peers_clone.lock().await;
                peers.push(get_value("Add peer address: "));
                true
            }
        });

        peer_page.add("0", "Continue" , || {
            async {
                false
            }
        });
        peer_page.run_menu().await;
    }

    pub async fn copy_db_backup(&mut self) {
        self.db = None;
        Self::copy_dir_all(Path::new("amanah.db"), Path::new("backup/")).unwrap();
        let db_path = "amanah.db";
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        let db = DB::open(&db_opts, db_path).unwrap();
        self.db = Some(db);
    }

    async fn sync_chain(mut chain: Chain) -> Chain {
        // open the db and check the height

        let mut height = chain.height;
        let mut max_addr = String::new();
        for addr in chain.node.peer_server_addr.lock().await.iter() {
            let response = reqwest::get(format!("http://{addr}/len"))
                .await
                .unwrap()
                .json::<Len>()
                .await
                .unwrap();
            println!("{addr}: {:?} => len =  {:?}", response.uuid, response.len);
            if height < response.len {
                height = response.len;
                max_addr = addr.clone();
            }
        }

        println!("max block heigh = {height}");

        // archive the db
        let response = reqwest::get(format!("http://{max_addr}/archive_db"))
            .await
            .unwrap()
            .json::<Msg>()
            .await
            .unwrap();

        if response.status == 400 {
            println!("Sync failed");
            return chain;
        }

        chain.db = None; // drop current db
        let (db, height) = Self::download_and_extract(max_addr).await;
        chain.height = height;
        chain.db = Some(db); // drop current db
        chain
    }


    fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), &'static str> {
        use std::fs;
        if src.is_dir() {
            // Create the destination directory if it doesn't exist
            fs::create_dir_all(dst).unwrap();

            // Iterate over the contents of the source directory
            for entry in fs::read_dir(src).unwrap() {
                let entry = entry.unwrap();
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());

                if src_path.is_dir() {
                    // Recursively copy subdirectories
                    Self::copy_dir_all(&src_path, &dst_path)?;
                } else {
                    // Copy files
                    if let Err(e) = fs::copy(&src_path, &dst_path) {
                        eprintln!("Erro while copying : {e}")
                    }
                }
            }
        } else {
            // If the source is not a directory, just copy the file
            if let Err(e) = fs::copy(src, dst) {
                eprintln!("Erro while copying : {e}")
            }
        }

        Ok(())
    }

   async fn download_and_extract(addr: String) -> (DB, u32) {
        // let current_dir = env::current_dir().unwrap();
        // let file_dir = format!("{}\\amanah.db\\",current_dir.display());
        // println!("{:?}",current_dir);

        // delete the current db 
        if let Err(e) = fs::remove_file("tmp/db.zip") {
            eprintln!("Couldn't able to remove db : {e}");
        }

        // create tmp dir
        if let Err(e) = fs::create_dir("tmp") {
            eprintln!("Couldn't able to create tmp directory: {e}");
        }
        {
            // URL of the file
            println!("Downloaded started.....");
            let url = format!("http://{}/static/db.zip",addr);

            // Send GET request
            let response = reqwest::get(url).await.unwrap();

            // Open a file to write the downloaded content
            let mut file = File::create("tmp/db.zip").unwrap();

            // Copy the content from the response to the file
            let content = response.bytes().await.unwrap();
            copy(&mut content.as_ref(), &mut file).unwrap();
            std::mem::drop(file);
        }

        // delete the tmp/amanah.db
        if let Err(e) = fs::remove_dir_all("tmp/backup") {
            eprintln!("Couldn't able to remove the db: {e}");
        }

        // extract the downloaded file
        use std::io::{self, Write};
        let output = Command::new("7z")
            .args(["x", "tmp\\db.zip", "-otmp\\"])
            .output().expect("Failed to execute the zip command");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        // // delete the zip
        if let Err(e) = fs::remove_file("tmp/db.zip") {
            eprintln!("Couldn't able to remove the zip: {e}");
        }

        // delete the amanah db
        if let Err(e) = fs::remove_dir_all("amanah.db") {
            eprintln!("Couldn't able to remove the db: {e}");
        }

        // create amanah.db dir
        if let Err(e) = fs::create_dir("amanah.db") {
            eprintln!("Couldn't able to create amanah.db directory: {}", e);
        }

        Self::copy_dir_all(Path::new("tmp/backup"), Path::new("amanah.db")).unwrap();
        let db_path = "amanah.db";
        let db = DB::open_default(&db_path).unwrap();
        let height = match db.get("height") { // do not convert to bytes
            Ok(Some(height)) => {
                let digits: [u8; 4] = height.try_into().unwrap();
                u32::from_be_bytes(digits)
            },
            _ => 0
        };
        println!("Height = {height}");
        println!("File downloaded successfully!");
        (db, height)
    } 

    pub async fn add_transaction(&mut self, transaction: Transaction) {
        self.curr_trans.insert(transaction.transaction_id.clone(), transaction);
    }

    pub async fn add_block(&mut self, block: Block) -> bool {
        println!("{:#?} going to add..", &block);

        let block_hash = Chain::hash(&block.header);

        if block.header.pre_hash !=  self.last_hash().await.unwrap() {
            println!("Invalid block. Failed to add the block");
            return false;
        }

        if self.db.as_mut().unwrap().put(block_hash.as_bytes(), serde_json::to_string(&block).unwrap().as_bytes()).is_err() {
            return false;
        }

        if self.db.as_mut().unwrap().put(self.height.to_be_bytes(), block_hash.as_bytes()).is_err() {
            return false;
        }
        
        if self.db.as_mut().unwrap().put("height", (self.height + 1).to_be_bytes()).is_err() {
            return false;
        }

        self.height += 1;
        self.db.as_mut().unwrap().flush().expect("Failed to add the data to the db");

        println!("removing the added transactions.....");
        println!("transactions already added: {:?}", block.transactions);
        println!("transactions to be removed: {:?}", self.curr_trans);
        for (id,_) in &block.transactions {
            self.curr_trans.remove(id);
        }
        println!("balance transaction: {:?}", self.curr_trans);

        true
    }

    pub async fn add_peer(&self, peer_addr: String) {
        self.node.add_peer(peer_addr).await;
    }

    pub async fn get_height(&mut self) -> u32 {
        match self.db.as_mut().expect("DB not found").get("height") { // do not convert to bytes
            Ok(Some(height)) => {
                let digits: [u8; 4] = height.try_into().unwrap();
                u32::from_be_bytes(digits)
            },
            _ => 0
        }
    }

    pub async fn reveal_chain(&mut self) {
        for i in 0..self.height {
            let block = self.get_block_by_index(i).await.unwrap();
            println!("Block hash: {}",Chain::hash(&block.header));
            println!("{:#?}", block)
        }
    }

    pub fn new_transaction(&mut self, sender: String, receiver: String, amount: f32) -> bool {
        let transaction_id = Uuid::new_v4();
        let transaction = Transaction {
            sender,
            receiver,
            amount,
            transaction_id: transaction_id.to_string(),
        };
        let trans_hash = Chain::hash(&transaction);
        self.curr_trans.insert(transaction_id.to_string(), transaction.clone());

        if let Err(e) = self.node.msg_outgoing_tx.send(Message {
            uuid: self.uuid.to_string(),
            block: None,
            transaction: Some(transaction),
            message_hash: trans_hash,
        }) {
            println!("Cannot broadcast the block due to {e}: ")
        };
        true
    }

    pub async fn last_hash(&mut self) -> Result<String, &str> {
        if self.height == 0 {
            return Ok(String::from_utf8(vec![48; 64]).unwrap());
        }
        if let Ok(hash) = self.get_hash_by_index(self.height - 1).await {
            Ok(hash)
        } else {
            Err("Hash not found for current index")
        }
    }

    pub async fn get_block_by_index(&mut self, index: u32) -> Result<Block, &str> {
        if let Ok(hash) = self.get_hash_by_index(index).await {
            match self.get_block_by_hash(hash).await {
                Ok(block) => Ok(block),
                Err(_) => Err("Block not found"),
            }
        } else {
            Err("Block not found")
        }
    }

    pub async fn get_hash_by_index(&mut self, index: u32) -> Result<String, &str>{
        match self.db.as_mut().expect("DB not found").get(index.to_be_bytes()) {
            Ok(Some(hash)) => {
                Ok(String::from_utf8(hash).unwrap())
            },
            Ok(None) => {
                Err("Block not found")
            },
            Err(_) => Err("operational error occured")
        }
    }

    pub async fn get_block_by_hash(&mut self, hash: String) -> Result<Block, &'static str> {
        match self.db.as_mut().expect("DB not found").get(hash.as_bytes()) {
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

    pub fn update_miner_address(&mut self, miner_address: String) {
        self.miner_addr = miner_address;
    }

    pub fn update_reward(&mut self, reward: f32) -> bool {
        self.reward = reward;
        true
    }

    pub async fn generate_new_block(&mut self) -> bool {
        println!("current transactons = {:?}", self.curr_trans);
        if !(self.height == 0) && self.curr_trans.is_empty() {
            println!("No transaction to add!!");
            return false;
        }

        self.miner_addr = if self.miner_addr.is_empty() {
            get_value("Enter the miner address: ")
        } else {
            self.miner_addr.clone()
        };

        let header = Blockheader {
            timestamp: Utc::now().timestamp_millis(),
            nonce: 0,
            pre_hash: self.last_hash().await.unwrap(),
            merkle: String::new(),
            difficulty: self.difficulty,
        };

        let transaction_id = Uuid::new_v4();
        let reward_trans = Transaction {
            sender: String::from("Root"),
            receiver: self.miner_addr.clone(),
            amount: self.reward,
            transaction_id: transaction_id.to_string()
        };

        let mut block = Block {
            header: header.clone(),
            count: 0,
            transactions: HashMap::new(),
        };

        block.transactions.insert(reward_trans.transaction_id.clone(), reward_trans);
        for (id, transactions) in self.curr_trans.clone().into_iter() {
            block.transactions.insert(id.clone(), transactions);
        }

        self.curr_trans = HashMap::new();

        block.count = block.transactions.len() as u32;

        let transaction_vec: Vec<_> = block.transactions.values().collect();
        block.header.merkle = Chain::get_merkle(transaction_vec);
        Chain::proof_of_work(&mut block.header);

        println!("{:#?}", &block);

        let block_hash = Chain::hash(&block.header);

        if self.db.as_mut().unwrap().put(block_hash.as_bytes(), serde_json::to_string(&block).unwrap().as_bytes()).is_err() {
            return false;
        }

        if self.db.as_mut().unwrap().put(self.height.to_be_bytes(), block_hash.as_bytes()).is_err() {
            return false;
        }
        
        if self.db.as_mut().unwrap().put("height", (self.height + 1).to_be_bytes()).is_err() {
            return false;
        }

        self.height += 1;
        self.db.as_mut().unwrap().flush().expect("Failed to add the data to the db");
        if let Err(e) = self.node.msg_outgoing_tx.send(Message {
            uuid: self.uuid.to_string(),
            // msg_id: 0,
            block: Some(block),
            transaction: None,
            message_hash: block_hash,
        }) {
            println!("Cannot broadcast the block due to {e}: ")
        };
        true
    }

    fn get_merkle(curr_trans: Vec<&Transaction>) -> String {
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