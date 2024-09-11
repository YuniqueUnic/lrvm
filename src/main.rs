pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

extern crate nom;

fn main() {
    let mut repl = repl::REPL::new();

    repl.run();
}
