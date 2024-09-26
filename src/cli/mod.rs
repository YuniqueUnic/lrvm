use clap_derive::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// Path to the .iasm or .ir file to run
    #[arg(short('f'), long)]
    pub file: Option<String>,

    /// Root directory where the lrvm VM should store its data. Defaults to /var/lib/lrvm.
    #[arg(long)]
    pub data_root_dir: Option<String>,

    /// An alias that can be used to refer to a running VM across a network
    #[arg(short('n'), long)]
    pub alias: Option<String>,

    /// Number of OS threads the VM will utilize
    #[arg(short('t'), long)]
    pub threads: Option<usize>,

    /// Enables the remote server component of lrvm VM
    #[arg(short('r'), long)]
    pub enable_remote_access: bool,

    /// Which address lrvm should listen for remote connections on. Defaults to "127.0.0.1".
    #[arg(short('o'), long("bind-host"))]
    pub listen_host: Option<String>,

    /// Which port lrvm should listen for remote connections on. Defaults to 65201.
    #[arg(short('p'), long("bind-port"))]
    pub listen_port: Option<String>,

    ///  Which address Iridium should listen for remote connections on from other Iridium VMs. Defaults to "127.0.0.1".
    #[arg(short('O'), long("server-bind-host"))]
    pub server_listen_host: Option<String>,

    /// Which port Iridium should listen for remote connections on from other Iridium VMs. Defaults to 65211.
    #[arg(short('P'), long("server-bind-port"))]
    pub server_listen_port: Option<String>,

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
