use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self},
};

use crate::util::display;

use super::NodeAlias;

#[derive(Debug)]
pub struct ClusterClient {
    alias: Option<NodeAlias>,
    // 用 BufReader 包装流，使其更容易读取
    pub reader: BufReader<TcpStream>,
    // 用 BufWriter 包装流，使其更容易写入
    writer: BufWriter<TcpStream>,
    // 这些是标准 mpsc 通道。
    // 我们将启动一个线程，监视此通道上来自我们应用程序其他部分的消息
    // 被发送到 ClusterClient
    rx: Option<Arc<Mutex<Receiver<String>>>>,
    // 如果有东西想要发送东西给这个客户端，它们可以克隆 `tx` 通道。
    _tx: Option<Arc<Mutex<Sender<String>>>>,
    raw_stream: TcpStream,
}

impl ClusterClient {
    pub fn new(raw_stream: TcpStream) -> Self {
        let (tx, rx) = mpsc::channel::<String>();
        let reader = raw_stream.try_clone().unwrap();
        let writer = raw_stream.try_clone().unwrap();
        ClusterClient {
            alias: None,
            reader: { BufReader::new(reader) },
            writer: { BufWriter::new(writer) },
            rx: Some(Arc::new(Mutex::new(rx))),
            _tx: Some(Arc::new(Mutex::new(tx))),
            raw_stream,
        }
    }

    pub fn run(&mut self) {
        // 在后台线程中启动 recv_loop
        self.recv_loop();
        let mut buf = String::new();
        // 循环处理传入数据，等待数据。
        loop {
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    buf.trim_end();
                },
                Err(e) => {
                    display::e_writeout(&format!("Error receiving: {:#?}", e));
                },
            }
        }
    }

    pub fn send_hello(&mut self) {
        let alias = self.alias.clone();
        let alias = alias.unwrap();
        if self.raw_stream.write(&alias.as_bytes()).is_ok() {
            display::writeout("Hello sent!");
        } else {
            display::e_writeout("Error sending hello!");
        }
    }

    pub fn with_alias(mut self, alias: NodeAlias) -> Self {
        self.alias = Some(alias);
        self
    }

    /// 这是一个后台循环，监视 mpsc 通道上的消息。
    /// 当它收到一个消息时，它将其发送到 ClusterClient。
    fn recv_loop(&mut self) {
        // 我们取 rx 通道，以便我们可以将所有权移入线程并循环
        let chan = self.rx.take().unwrap();
        let mut writer = self.raw_stream.try_clone().unwrap();
        let _t = thread::spawn(move || loop {
            if let Ok(locked_rx) = chan.lock() {
                match locked_rx.recv() {
                    Ok(msg) => {
                        match writer.write_all(msg.as_bytes()) {
                            Ok(_) => {},
                            Err(e) => {
                                display::e_writeout(&format!("Error writing to client: {}", e));
                            },
                        };
                        match writer.flush() {
                            Ok(_) => {},
                            Err(e) => {
                                display::e_writeout(&format!("Error writing to client: {}", e));
                            },
                        }
                    },
                    Err(e) => {
                        display::e_writeout(&format!("Error writing to client: {}", e));
                    },
                }
            }
        });
    }

    #[allow(dead_code)]
    /// 将消息作为字节写入连接的 ClusterClient
    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(e) => {
                    display::e_writeout(&format!("Error flushing to client: {}", e));
                    false
                },
            },
            Err(e) => {
                display::e_writeout(&format!("Error writing to client: {}", e));
                false
            },
        }
    }
}
