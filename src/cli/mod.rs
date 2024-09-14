use clap_derive::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// Path to the .iasm or .ir file to run
    #[arg(short, long)]
    pub file: Option<String>,

    /// test the cli args
    #[arg(short, long)]
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
