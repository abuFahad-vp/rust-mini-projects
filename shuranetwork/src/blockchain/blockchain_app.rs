use crate::blockchain::blockchain_core::Chain;
use crate::blockchain::blockchain_tui;
use crate::blockchain::blockchain_rest;
use crate::utils::get_value;

pub async fn blockchain_app() {

    // port assigning
    let port_node = get_value("Enter port number for node: ");
    let port_server = get_value("Enter port number for server: ");
    let port_node = port_node.parse::<u16>().unwrap();
    let port_server    = port_server.parse::<u16>().unwrap();

    // blockchain initialization
    let chain = Chain::new(port_node, port_server).await;
    let msg_incoming_rx = chain.msg_incoming_rx.clone();
    let msg_outgoing_tx = chain.msg_outgoing_tx.clone();

    let chain = std::sync::Arc::new(tokio::sync::Mutex::new(chain));


    let chain_clone = chain.clone();
    tokio::spawn(async move {
        blockchain_rest::blockchain_app_run(chain_clone, port_server).await.unwrap();
    });

    let chain_clone = chain.clone();
    tokio::spawn(async move {
        // let mut chain = chain_clone.lock().await;
        let reciever = msg_incoming_rx;
        while let Ok(msg) = reciever.recv() {
            println!("{msg:?}");
            if let Some(block) = &msg.block {
                let mut chain = chain_clone.lock().await;
                chain.add_block(block.clone()).await;
                println!("block recieved")
            }
            if let Some(transaction) = &msg.transaction {
                let mut chain = chain_clone.lock().await;
                chain.add_transaction(transaction.clone()).await;
                println!("transaction recieved")
            }
            if let Err(e) = msg_outgoing_tx.send(msg) {
                eprintln!("Cannot transmit the message to internal reciever: {e}")
            }
        }
    });

    let chain_clone = chain.clone();
    blockchain_tui::blockchain_app_run(chain_clone, port_node).await.run_menu().await;
}
