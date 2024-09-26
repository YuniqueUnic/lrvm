use std::{
    io::Read,
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
    thread,
};

use crate::util::display;

use super::{client::ClusterClient, manager::Manager};

pub fn listen(addr: SocketAddr, connection_manager: Arc<RwLock<Manager>>) {
    display::writeout("Initializing Cluster server...");
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let cmgr = connection_manager.clone();
        display::writeout("New Node connected!");
        let stream = stream.unwrap();
        thread::spawn(move || {
            let mut buf = [0; 1024];
            let mut client = ClusterClient::new(stream);
            // Once this call succeeds, we'll hopefully have the node alias in the string buffer
            let bytes_read = client.reader.read(&mut buf).unwrap();
            let alias = String::from_utf8_lossy(&buf[0..bytes_read]);
            let mut cmgr_lock = cmgr.write().unwrap();
            cmgr_lock.add_client(alias.into_owned(), client);
            // let mut client = ClusterClient::new(stream);
            // client.run();
        });
    }
}
