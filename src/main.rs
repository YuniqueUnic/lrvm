pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

extern crate nom;

/// Starts the REPL that will run until the user kill it;
fn main() {
    let mut repl = repl::REPL::new();

    repl.run();
}
