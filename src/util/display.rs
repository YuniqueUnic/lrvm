use log::{error, info};

pub fn e_writeout(msg: &str) {
    let msg = format!("[Error]: {}", msg);
    error!("{}", msg);
    eprintln!("{}", msg);
}

pub fn writeout(msg: &str) {
    let msg = format!("[ Info]: {}", msg);
    info!("{}", msg);
    println!("{}", msg);
}
