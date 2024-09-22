pub mod command_parser;

use command_parser::CommandParser;

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

            let history_copy = buffer.clone();

            self.command_buffer.push(history_copy);

            if buffer.starts_with("!") {
                self.execute_command(&buffer);
            } else {
                let program = match program(&buffer) {
                    Ok((_reminder, program)) => program,
                    Err(e) => {
                        eprintln!("Unable to parse input: {}", e);
                        continue;
                    },
                };

                self.vm
                    .program
                    .append(&mut program.to_bytes(&self.asm.symbols));

                self.vm.run_once();
            }
        }
    }

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!load_file" => {
                let contents = self.get_data_from_load();
                self.load_file(&args[1..], &contents);
            },
            "!spawn" => {
                let contents = self.get_data_from_load();
                self.spawn(&args[1..], &contents);
            },
            _ => {
                println!("Unknown command: {:?}", args);
            },
        }
    }

    fn quit(&mut self, args: &[&str]) {
        println!("Farewell ! Have a great day.");
        std::process::exit(0);
    }
    fn history(&mut self, args: &[&str]) {
        for command in &self.command_buffer {
            println!("{}", command);
        }
    }
    fn program(&mut self, args: &[&str]) {
        println!("Listing instructions currently in vm's program vector!");
        for instruction in &self.vm.program {
            println!("{}", instruction);
        }
        println!("End of Program Listing");
    }

    fn registers(&mut self, args: &[&str]) {
        println!("Listing registers and all contents:");
        println!("{:#?}", &self.vm.registers);
        println!("End of registers Listing");
    }

    fn load_file(&mut self, args: &[&str], data_from_file: &Option<String>) {
        if let Some(contents) = data_from_file {
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

    fn spawn(&mut self, args: &[&str], data_from_file: &Option<String>) {
        if let Some(contents) = data_from_file {
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

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, path::PathBuf};

    use io::Error;

    use super::*;

    /// 读取文件内容到字符串
    fn read_file_to_string(file_path: &str) -> Result<String, Error> {
        let mut file = match OpenOptions::new().read(true).open(file_path) {
            Ok(content) => content,
            Err(e) => return Err(e),
        };

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => Ok(contents),
            Err(e) => Err(e),
        }
    }

    /// 获取给定路径的绝对路径
    ///
    /// 本函数通过获取当前工作目录并与给定的路径字符串拼接，来生成给定路径的绝对路径。
    /// 如果无法获取当前工作目录，则会引发 panic。
    ///
    /// # 参数
    /// * `path` - 一个表示相对当前工作目录的路径字符串
    ///
    /// # 返回值
    /// * 返回一个`PathBuf`对象，表示拼接后的绝对路径
    ///
    /// # 错误处理
    /// * 如果无法获取当前工作目录，函数将 panic
    fn get_absolute_path(path: &str) -> PathBuf {
        // 尝试获取当前工作目录
        let current_dir = if let Ok(dir) = std::env::current_dir() {
            dir
        } else {
            // 如果无法获取当前工作目录，输出错误信息并终止程序
            panic!("Unable to get current directory")
        };

        // 将当前工作目录与给定路径字符串拼接，生成绝对路径
        let path = current_dir.join(path);
        path
    }

    #[test]
    fn test_spawn() {
        let test_file = get_absolute_path("docs/examples/hlt.iasm");

        let contents = match read_file_to_string(test_file.to_str().unwrap()) {
            Ok(content) => Some(content),
            Err(err) => panic!("Unable to read file:{}", err),
        };

        let mut repl = REPL::new();
        repl.spawn(&[""], &contents);
        assert!(repl.asm.errors.is_empty());
    }
}
