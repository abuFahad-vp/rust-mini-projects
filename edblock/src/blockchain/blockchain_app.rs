use std::sync::Mutex;
use rocksdb::{Options, DB};
use crate::blockchain::blockchain_core::Chain;
use crate::blockchain::blockchain_tui;
use crate::blockchain::blockchain_rest;
use crate::template::{self};
use crate::utils::get_value;

pub async fn blockchain_app() {

    // port assigning
    let port_node = get_value("Enter port number for node: ");
    let port_server = get_value("Enter port number for server: ");
    let port_node = port_node.parse::<u16>().unwrap();
    let port_server    = port_server.parse::<u16>().unwrap();

    // blockchain initialization
    let db_path = "amanah.db";
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    let db = DB::open(&db_opts, db_path).unwrap();
    let chain = std::sync::Arc::new(tokio::sync::Mutex::new(
        Chain::new(db, port_node).await
    ));
    // getting the peers
    let peers = std::sync::Arc::new(Mutex::new(Vec::new()));
    get_peers(peers.clone()).await;

    {
        let chain_clone = chain.clone();
        let chain_clone = chain_clone.lock().await;
        for peer in peers.lock().unwrap().iter() {
            chain_clone.add_peer(peer.to_string()).await;
            println!("peers = {}",peer);
        }
    }


    let chain_clone = chain.clone();
    tokio::spawn(async move {
        blockchain_rest::blockchain_app_run(chain_clone, port_server).await.unwrap();
    });

    blockchain_tui::blockchain_app_run(chain.clone(), port_node).run_menu().await;
}

async fn get_peers(peers: std::sync::Arc<Mutex<Vec<String>>>) {

    // adding peer
    let peers_clone = peers.clone();
    let mut peer_page = template::MenuBuilder::new();

    peer_page.add("1", "Add peer", move || {
        let peers_clone = peers_clone.clone();
        async move {
            let mut peers = peers_clone.lock().unwrap();
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
