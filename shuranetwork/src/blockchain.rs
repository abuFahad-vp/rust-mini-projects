use tokio::sync::Mutex;

use blockchain_core::Chain;

pub mod blockchain_core;
pub mod blockchain_app;
pub mod blockchain_tui;
pub mod blockchain_rest;
pub mod peer_network;

type SharedChain = std::sync::Arc<Mutex<Chain>>;