use clap_derive::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// Path to the .iasm or .ir file to run
    #[arg(short('f'), long)]
    pub file: Option<String>,

    /// Number of OS threads the VM will utilize
    #[arg(short('t'), long)]
    pub threads: Option<usize>,

    /// Enables the remote server component of lrvm VM
    #[arg(short('r'), long)]
    pub enable_remote_access: bool,

    /// Which address lrvm should listen for remote connections on. Defaults to "127.0.0.1".
    #[arg(short('h'), long("bind-host"))]
    pub listen_host: Option<String>,

    /// Which port lrvm should listen for remote connections on. Defaults to 2244.
    #[arg(short('p'), long("bind-port"))]
    pub listen_port: Option<String>,

    /// test the cli args
    #[arg(long)]
    pub test: bool,

    /// The command to run
    #[command(subcommand)]
    pub command: Option<Vers>,
}

#[derive(Subcommand)]
pub enum Vers {
    /// Runs the file
    Run,

    /// Prints the text
    Print(InnertText),
}

#[derive(Args)]
pub struct InnertText {
    /// The text to print
    pub content: Option<String>,
}
