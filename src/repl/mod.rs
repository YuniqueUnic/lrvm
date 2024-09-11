use crate::vm::VM;

use std;
use std::io;
use std::io::Write;
use std::num::ParseIntError;

pub struct REPL {
    command_buffer: Vec<String>,
    // the VM the REPL will use to execute code
    vm: VM,
}

impl REPL {
    /// Creates and returns a new assembly repl
    pub fn new() -> REPL {
        REPL {
            command_buffer: vec![],
            vm: VM::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to lrvm! Let's be productive.");
        loop {
            let mut buffer = String::new();

            let stdin = io::stdin();

            print!(">>> ");
            io::stdout().flush().expect("unable to flush stdout");

            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            let buffer = buffer.trim();

            self.command_buffer.push(buffer.to_string());

            match buffer {
                ".quit" => {
                    println!("Farewell ! Have a great day.");
                    std::process::exit(0);
                },
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{}", command);
                    }
                },
                ".program" => {
                    println!("Listing instructions currently in vm's program vector!");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                },
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", &self.vm.registers);
                    println!("End of registers Listing");
                },
                _ => {
                    let results = self.parse_hex(buffer);
                    match results {
                        Ok(bytes) => {
                            for byte in bytes {
                                self.vm.add_byte(byte)
                            }
                        },
                        Err(e) => {
                            eprintln!("Unable to decode hex string, Please enter 4 groups of 2 hex characters.\n Error: {e}")
                        },
                    }
                    self.vm.run_once();
                },
            }
        }
    }

    fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split: Vec<&str> = i.split(" ").collect::<Vec<&str>>();

        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);

            match byte {
                Ok(res) => {
                    results.push(res);
                },
                Err(e) => {
                    return Err(e);
                },
            }
        }

        Ok(results)
    }
}
