
use local_ip_address::local_ip;
use crate::template;
use crate::blockchain;
use crate::Args;

pub fn starter_page(args: Args) {
    let header = format!("

    Welcome to shuranetwork!!
    IP ADDRESS: {}:{}
    NODE STATUS: OFFLINE

", local_ip().expect("Failed to get the local ip. Internal Error"), args.port);
    let mut start_page = template::MenuBuilder::new();
    start_page.set_header(header);
    start_page.add("1", "Go Online", || {
        blockchain::blockchain_ui::blockchain_ui_page();
        true
    });

    start_page.add("2", "Add peer addresses", || {
        true
    });

    start_page.add("0", "Exit", || {
        false
    });
    start_page.run_menu();
}