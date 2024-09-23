use std::{net::TcpListener, thread};

use crate::remote::client::Client;

pub struct Server {
    bind_hostname: String,
    bind_port: String,
}

impl Server {
    pub fn new(bind_hostname: String, bind_port: String) -> Self {
        Server {
            bind_hostname,
            bind_port,
        }
    }

    pub fn listen(&mut self) {
        println!("Initializing TCP Server...");

        let address = format!("{}:{}", self.bind_hostname, self.bind_port);
        let listener = TcpListener::bind(address).unwrap();

        for stream in listener.incoming() {
            if let Ok(s) = stream {
                thread::spawn(|| {
                    let mut client = Client::new(s);
                    client.run();
                });
            }
        }
    }
}
