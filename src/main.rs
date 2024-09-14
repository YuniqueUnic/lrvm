use std::{fs::File, io::Read, path::Path};

use clap::Parser;
use cli::CLI;

pub mod assembler;
pub mod cli;
pub mod instruction;
pub mod repl;
pub mod vm;

extern crate nom;

/// Starts the REPL that will run until the user kills it.
fn main() {
    let cli = CLI::parse();

    if let Some(command) = &cli.command {
        match command {
            cli::Vers::Run => {
                println!("RUNNING...");
            },
            cli::Vers::Print(v) => {
                if let Some(text) = &v.content {
                    println!("The user text: {:?}", text);
                }
            },
        }
    }

    if let Some(filename) = &cli.file {
        let program = read_file(&filename);
        let mut asm = assembler::Assembler::new();
        let mut vm = vm::VM::new();
        if let Some(p) = asm.assemble(&program) {
            vm.add_bytes(p);
            vm.run();
        }
    } else {
        start_repl();
    }
}

fn read_file(filename: &str) -> String {
    let filename = Path::new(filename);
    let mut fh = File::open(filename).expect("File not found");
    let mut contents = String::new();
    fh.read_to_string(&mut contents)
        .expect("Error reading file");
    contents
}

fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}
