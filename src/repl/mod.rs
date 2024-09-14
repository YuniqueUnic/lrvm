use crate::assembler::program_parser::program;
use crate::assembler::Assembler;
use crate::scheduler::Scheduler;
use crate::vm::VM;

use std;
use std::fs::File;
use std::io::{self, stdin};
use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::Path;

pub struct REPL {
    command_buffer: Vec<String>,
    // the VM the REPL will use to execute code
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
}

impl REPL {
    /// Creates and returns a new assembly repl
    pub fn new() -> REPL {
        REPL {
            command_buffer: vec![],
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
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
                ".quit" => self.quit(),
                ".history" => self.history(),
                ".program" => self.program(),
                ".registers" => self.registers(),
                ".load_file" => self.load_file(),
                ".spawn" => self.spawn(),
                _ => {
                    let parsed_program = program(buffer);
                    if !parsed_program.is_ok() {
                        eprintln!("Unable to parse input");
                        continue;
                    }

                    let (_, result) = parsed_program.unwrap();

                    let bytecode = result.to_bytes(&self.asm.symbols);

                    for byte in bytecode {
                        self.vm.add_byte(byte);
                    }

                    self.vm.run_once();
                },
            }
        }
    }

    fn quit(&mut self) {
        println!("Farewell ! Have a great day.");
        std::process::exit(0);
    }
    fn history(&mut self) {
        for command in &self.command_buffer {
            println!("{}", command);
        }
    }
    fn program(&mut self) {
        println!("Listing instructions currently in vm's program vector!");
        for instruction in &self.vm.program {
            println!("{}", instruction);
        }
        println!("End of Program Listing");
    }

    fn registers(&mut self) {
        println!("Listing registers and all contents:");
        println!("{:#?}", &self.vm.registers);
        println!("End of registers Listing");
    }

    fn load_file(&mut self) {
        if let Some(contents) = self.get_data_from_load() {
            let program = match program(&contents) {
                Ok((_reminder, program)) => program,
                Err(e) => {
                    eprintln!("Unable to parse input: {:?}", e);
                    return;
                },
            };
            self.vm
                .program
                .append(&mut program.to_bytes(&self.asm.symbols));
        }
    }

    fn spawn(&mut self) {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    println!("Sending assembled program to VM");
                    self.vm.program.append(&mut assembled_program);
                    println!("{:#?}", self.vm.program);
                    self.scheduler.get_thread(self.vm.clone());
                },
                Err(errors) => {
                    for error in errors {
                        println!("Unable to parse input: {}", error);
                    }
                },
            }
        } else {
        }
    }

    fn get_data_from_load(&mut self) -> Option<String> {
        print!("Please enter the path to the file you wish to load: ");
        io::stdout().flush().expect("Unable to flush stdout");
        let mut tmp = String::new();
        stdin()
            .read_line(&mut tmp)
            .expect("Unable to read line from user");

        let tmp = tmp.trim();
        let filename = Path::new(&tmp);

        let mut f = match File::open(Path::new(&filename)) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Unable to open file: {}", e);
                return None;
            },
        };

        let mut contents = String::new();

        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                eprintln!("Unable to read file: {}", e);
                None
            },
        }
    }

    #[allow(dead_code)]
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
