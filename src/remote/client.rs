use std::io::{BufRead, BufWriter, Write};
use std::thread;
use std::{io::BufReader, net::TcpStream};

use crate::repl::{self};
use crate::vm::VM;

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    raw_stream: TcpStream,
    repl: repl::REPL,
}

impl Client {
    pub fn new(raw_stream: TcpStream) -> Self {
        let reader = raw_stream.try_clone().unwrap();
        let writer = raw_stream.try_clone().unwrap();
        let repl = repl::REPL::new(VM::new());

        Client {
            reader: { BufReader::new(reader) },
            writer: { BufWriter::new(writer) },
            raw_stream,
            repl,
        }
    }

    fn recv_loop(&mut self) {
        let rx = self.repl.rx_pipe.take();
        // TODO: Make this safer on unwrap
        let mut writer = self.raw_stream.try_clone().unwrap();
        let _t = thread::spawn(move || {
            let channel = rx.unwrap();
            loop {
                match channel.recv() {
                    Ok(msg) => {
                        let _ = writer.write_all(msg.as_bytes());
                        let _ = writer.flush();
                    },
                    Err(e) => {
                        let _ = writer.write_all(
                            format!("Unable to recieve data in recv_loop: {:?}", e).as_bytes(),
                        );
                        let _ = writer.flush();
                    },
                }
            }
        });
    }

    pub fn run(&mut self) {
        self.recv_loop();
        // let mut buf = String::new();  // remote msg will be accumulated, and only the top first msg will be handled
        let banner = format!("{}\n{}", repl::REMOTE_BANNER, repl::PROMPT);
        self.w(&banner);
        loop {
            let mut buf = String::new();
            match self.reader.read_line(&mut buf) {
                Ok(_) => {
                    self.repl.run_single(&buf.trim_end());
                },
                Err(e) => {
                    eprintln!("Error receiving: {:#?}", e);
                },
            }
        }
    }

    fn write_prompt(&mut self) {
        self.w(repl::PROMPT);
    }

    fn w(&mut self, msg: &str) -> bool {
        match self.writer.write_all(msg.as_bytes()) {
            Ok(_) => match self.writer.flush() {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Error flushing to client: {}", e);
                    false
                },
            },
            Err(e) => {
                eprintln!("Error flushing to client: {}", e);
                false
            },
        }
    }
}
