use std::sync::Arc;
use tokio::sync::Mutex;
use local_ip_address::local_ip;
use crate::blockchain::blockchain_core::Chain;
use crate::template::{self, MenuBuilder};
use crate::utils::get_value;

use super::SharedChain;

pub async fn blockchain_app_run(chain: SharedChain, port: u16) -> MenuBuilder {
    let header = format!("
    Welcome to shuranetwork!!
    IP ADDRESS: {}:{}
    ", local_ip().expect("Failed to get the local ip. Internal Error"), port);

    let mut blockchain_page = template::MenuBuilder::new();
    blockchain_page.set_header(header);

    blockchain_page.add("0", "Exit", || {
        async {
            false
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("1", "New Transaction",
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                new_transaction(chain).await;
                true
            }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("2", "Mine block", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                mine_block(chain).await;
                true
            }
    }});

    let chain_clone = chain.clone();
    blockchain_page.add("3", "Change difficulty", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                change_difficulty(chain).await;
                true
            }
    }});

    let chain_clone = chain.clone();
    blockchain_page.add("4", "Change Reward", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                change_reward(chain).await;
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("5", "reveal chain", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let mut chain = chain_clone.lock().await;
                chain.reveal_chain().await;
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("6", "Show height", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let mut chain = chain_clone.lock().await;
                println!("height: {}",chain.get_height().await);
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("7", "Show hash by index", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                show_hash_by_index(chain).await;
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("8", "Change miner address", {
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                change_miner_address(chain).await;
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("9", "Add new peer address", {
        let chain_clone = chain_clone.clone();
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                let chain = chain.lock().await;
                chain.add_peer(get_value("Enter peer address: ")).await;
                true
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_page.add("10", "Show server peer addresses", {
        let chain_clone = chain_clone.clone();
        move || {
            let chain_clone = chain_clone.clone();
            async move {
                let chain = chain_clone.clone();
                let chain = chain.lock().await;
                for addr in chain.node.peer_server_addr.lock().await.iter() {
                    println!("{addr}")
                }
                true
            }
        }
    });
    blockchain_page
}

async fn show_hash_by_index(chain: Arc<Mutex<Chain>>) {

    let choice = get_value("Input index: ");

    let mut chain = chain.lock().await;
    if let Ok(choice) = choice.trim().parse::<u32>() {
        println!("hash of index {}: {:?}",choice, chain.get_hash_by_index(choice).await);
    } else {
        println!("Invalid input");
    }
}

async fn new_transaction(chain: Arc<Mutex<Chain>>) {
    let sender = get_value("Enter Sender Address: ");
    let reciever = get_value("Enter Reciever Address: ");
    let amount = get_value("Enter the amount: ");

    let mut chain = chain.lock().await;

    let res = chain.new_transaction(
        sender.to_string(),
        reciever.to_string(),
        amount.parse().unwrap()
    );

    match res {
        true => println!("Transaction added"),
        false => println!("Transaction failed"),
    }
}

async fn mine_block(chain: Arc<Mutex<Chain>>) {
    println!("Generating block...");
    let mut chain = chain.lock().await;
    let res = chain.generate_new_block().await;
    match res {
        true => println!("Block added successfully"),
        false => println!("Block failed to add")
    }
}

async fn change_difficulty(chain: Arc<Mutex<Chain>>) {

    let new_diff = get_value("Enter new difficulty: ");
    
    let mut chain = chain.lock().await;
    let res = chain.update_difficulty(new_diff.trim().parse().unwrap());
    match res {
        true => println!("Updated Difficulty"),
        false => println!("Failed to update the difficulty")
    }
}

async fn change_reward(chain: Arc<Mutex<Chain>>) {
    let new_reward = get_value("Enter new reward: ");

    let mut chain = chain.lock().await;
    let res = chain.update_reward(new_reward.trim().parse().unwrap());
    match res {
        true => println!("Updated reward"),
        false => println!("Failed to update the reward")
    }
}
async fn change_miner_address(chain: Arc<Mutex<Chain>>) {
    let miner_addr = get_value("Enter new miner address: ");
    let mut chain = chain.lock().await;
    chain.update_miner_address(miner_addr);
}