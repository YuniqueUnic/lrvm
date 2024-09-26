use std::{fs::File, io::Read, path::Path, thread};

use clap::Parser;
use lrvm::{
    assembler,
    cli::{self, CLI},
    repl,
    util::display::{self},
    vm::{self, VM},
};

extern crate nom;
extern crate num_cpus;

static NODE_ID_FILENAME: &'static str = ".node_id";
static DEFAULT_NODE_LISTEN_HOST: &'static str = "127.0.0.1";
static DEFAULT_NODE_LISTEN_PORT: &'static str = "65211";
static DEFAULT_REMOTE_ACCESS_PORT: &'static str = "65201";

/// Starts the REPL that will run until the user kills it.
fn main() {
    env_logger::init();
    display::writeout("Starting logging!");

    let cli = CLI::parse();

    let data_root_dir = cli.data_root_dir.unwrap_or(String::from("/var/lib/lrvm/"));
    if make_directory(&data_root_dir).is_err() {
        display::writeout("There was an error creating the default root data directory");
        std::process::exit(1);
    };

    if cli.enable_remote_access {
        let host = cli.listen_host.unwrap_or(DEFAULT_NODE_LISTEN_HOST.into());
        let port = cli.listen_port.unwrap_or(DEFAULT_REMOTE_ACCESS_PORT.into());
        start_remote_server(host, port);
    }

    // Find or generate a unique node ID
    let alias = cli.alias.unwrap_or(String::new());

    display::writeout(&format!("Node ID is: {}", alias));

    let server_host = cli
        .server_listen_host
        .unwrap_or(DEFAULT_NODE_LISTEN_HOST.into());
    let server_port = cli
        .server_listen_port
        .unwrap_or(DEFAULT_NODE_LISTEN_PORT.into());

    let num_threads = match &cli.threads {
        Some(num) => *num,
        None => num_cpus::get(),
    };

    if let Some(command) = &cli.command {
        match command {
            cli::Vers::Run => {
                display::writeout("RUNNING...");
            },
            cli::Vers::Print(v) => {
                if let Some(text) = &v.content {
                    display::writeout(&format!("The user text: {:?}", text));
                }
            },
        }
    }

    if let Some(filename) = &cli.file {
        let program = read_file(&filename);
        let mut asm = assembler::Assembler::new();
        let mut vm = vm::VM::new()
            .with_alias(alias)
            .with_cluster_bind(server_host, server_port);
        vm.logical_cores = num_threads;
        if let Ok(p) = asm.assemble(&program) {
            vm.add_bytes(p);
            let events = vm.run();
            display::writeout("虚拟机事件");
            display::writeout("--------------------------");
            for event in &events {
                display::writeout(&format!("{:#?}", event));
            }
            std::process::exit(0);
        }
    } else {
        start_repl(alias, server_host, server_port);
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

fn start_remote_server(listen_host: String, listen_port: String) {
    let _t = std::thread::spawn(move || {
        let mut sh = lrvm::remote::server::Server::new(listen_host, listen_port);
        sh.listen();
    });
}

fn start_repl(alias: String, server_addr: String, server_port: String) {
    display::writeout(&format!("Spawning REPL with alias {}", alias));
    let vm = VM::new()
        .with_alias(alias)
        .with_cluster_bind(server_addr, server_port);
    let mut repl = repl::REPL::new(vm);
    let rx = repl.rx_pipe.take();
    thread::spawn(move || loop {
        match rx {
            Some(ref channel) => match channel.recv() {
                Ok(msg) => {
                    display::writeout(&msg);
                },
                Err(e) => {
                    display::e_writeout(&format!("Error receiving message: {}", e));
                },
            },
            None => {},
        }
    });
    repl.run();
}

fn make_directory(dir: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}
