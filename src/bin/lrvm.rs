use std::{fs::File, io::Read, path::Path};

use clap::Parser;
use lrvm::{
    assembler,
    cli::{self, CLI},
    repl, vm,
};

extern crate nom;
extern crate num_cpus;

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

    if cli.enable_remote_access {
        let host = &cli.listen_host.unwrap_or("127.0.0.1".into());
        let port = &cli.listen_port.unwrap_or("2244".into());
        start_remote_server(host, port);
    }

    let num_threads = match &cli.threads {
        Some(num) => *num,
        None => num_cpus::get(),
    };

    if let Some(filename) = &cli.file {
        let program = read_file(&filename);
        let mut asm = assembler::Assembler::new();
        let mut vm = vm::VM::new();
        vm.logical_cores = num_threads;
        if let Ok(p) = asm.assemble(&program) {
            vm.add_bytes(p);
            let events = vm.run();
            println!("虚拟机事件");
            println!("--------------------------");
            for event in &events {
                println!("{:#?}", event);
            }
            std::process::exit(0);
        }
    } else {
        start_repl();
    }
}

fn start_remote_server(listen_host: &str, listen_port: &str) {
    // let _t = std::thread::spawn(move ||{
    //     let mut sh =lrvm::remote
    // });
    todo!()
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
