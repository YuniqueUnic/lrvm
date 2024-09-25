pub mod command_parser;

use command_parser::CommandParser;

use crate::assembler::program_parser::program;
use crate::assembler::Assembler;
use crate::scheduler::Scheduler;
use crate::vm::VM;

use std::io::{self, stdin};
use std::io::{Stdin, Write};
use std::num::ParseIntError;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{self, vec};

const COMMAND_PREFIX: char = '!';

pub static REMOTE_BANNER: &'static str = "Welcome to lrvm! Let's be productive.";
pub static PROMPT: &'static str = ">>> ";

#[derive(Debug, Default)]
pub struct CommandManager {
    command_buffer: Vec<String>,
    offset: usize,
}

impl CommandManager {
    pub fn new() -> Self {
        CommandManager {
            command_buffer: vec![],
            offset: 0,
        }
    }

    pub fn push(&mut self, command: String) {
        self.command_buffer.push(command);
        self.offset += 1;
    }

    pub fn last_command(&mut self) -> String {
        if self.offset == 0 {
            self.currnet_command()
        } else {
            self.offset -= 1;
            self.currnet_command()
        }
    }

    pub fn currnet_command(&self) -> String {
        self.command_buffer[self.offset - 1].clone()
    }

    pub fn next_command(&mut self) -> String {
        self.offset += 1;
        self.currnet_command()
    }

    pub fn clear_all(&mut self) {
        self.command_buffer = vec![];
        self.offset = 0;
    }
}

pub struct REPL {
    command_manager: CommandManager,
    // the VM the REPL will use to execute code
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
    pub tx_pipe: Option<Box<Sender<String>>>,
    pub rx_pipe: Option<Box<Receiver<String>>>,
}

impl REPL {
    /// Creates and returns a new assembly repl
    pub fn new() -> REPL {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        REPL {
            command_manager: CommandManager::new(),
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
            tx_pipe: { Some(Box::new(tx)) },
            rx_pipe: { Some(Box::new(rx)) },
        }
    }

    pub fn send_prompt(&mut self) {
        match &self.tx_pipe {
            Some(pipe) => {
                let _ = pipe.send(format!("{}", PROMPT));
            },
            None => {
                println!("{}", PROMPT);
            },
        }
    }

    pub fn send_message(&mut self, msg: &str) {
        match &self.tx_pipe {
            Some(pipe) => {
                let _ = pipe.send(format!("{}\n", msg));
            },
            None => {
                println!("{}", msg);
            },
        }
    }

    pub fn run_single(&mut self, buffer: &str) -> Option<String> {
        if buffer.starts_with(COMMAND_PREFIX) {
            self.execute_command(&buffer);
            None
        } else {
            let program = match program(&buffer) {
                Ok((_reminder, program)) => Some(program),
                Err(e) => {
                    self.send_message(&format!("[Error]: Unable to parse input: {:?}", e));
                    self.send_prompt();
                    None
                },
            };
            match program {
                Some(p) => {
                    let mut bytes = p.to_bytes(&self.asm.symbols);
                    self.vm.program.append(&mut bytes);
                    self.vm.run_once();
                    self.send_prompt();
                    None
                },
                None => None,
            }
        }
    }

    pub fn run(&mut self) {
        self.write_local_loop();

        self.send_message(REMOTE_BANNER);
        self.send_prompt();

        loop {
            let mut buffer = String::new();

            let stdin = io::stdin();

            stdin
                .read_line(&mut buffer)
                .expect("[Error]: Unable to read line from user");

            let history_copy = String::from(buffer.trim());

            self.command_manager.push(history_copy);

            if buffer.starts_with(COMMAND_PREFIX) {
                self.execute_command(&buffer);
            } else {
                let program = match program(&buffer) {
                    Ok((_reminder, program)) => program,
                    Err(e) => {
                        self.send_message(&format!("Unable to parse input: {:?}", e));
                        self.send_prompt();
                        continue;
                    },
                };

                self.vm
                    .program
                    .append(&mut program.to_bytes(&self.asm.symbols));

                self.vm.run_once();
                self.send_prompt();
            }
        }
    }

    fn write_local_loop(&mut self) {
        let recv = self.rx_pipe.take();
        std::thread::spawn(move || loop {
            match recv {
                Some(ref pipe) => match pipe.recv() {
                    Ok(msg) => {
                        io::stdout()
                            .write(msg.as_bytes())
                            .expect("unable to write stdout");
                        io::stdout().flush().expect("unable to flush stdout");
                    },
                    Err(e) => {
                        let error = format!("Error: {:#?}", e);
                        io::stderr()
                            .write(error.as_bytes())
                            .expect("unable to write stdout");
                        io::stdout().flush().expect("unable to flush stdout");
                    },
                },
                None => {},
            }
        });
    }

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear" => self.clear(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => {
                let contents;

                match utils::aggreate_path(&args[1..]) {
                    Some(user_input_path) => {
                        let path = utils::is_valid_path(&user_input_path);
                        match path {
                            Some(valid_path) => {
                                contents = utils::get_data_from_load(valid_path);
                            },
                            None => {
                                contents = self.require_file_to_load();
                            },
                        }
                    },
                    None => {
                        contents = self.require_file_to_load();
                    },
                }

                self.load_file(&args[1..], &contents);
            },
            "!spawn" => {
                let contents;

                match utils::aggreate_path(&args[1..]) {
                    Some(user_input_path) => {
                        let path = utils::is_valid_path(&user_input_path);
                        match path {
                            Some(valid_path) => {
                                contents = utils::get_data_from_load(valid_path);
                            },
                            None => {
                                contents = self.require_file_to_load();
                            },
                        }
                    },
                    None => {
                        contents = self.require_file_to_load();
                    },
                }

                self.spawn(&args[1..], &contents);
            },
            _ => {
                self.send_message(&format!("Invalid command!: {}", args[0]));
                self.send_prompt();
            },
        }
    }

    fn quit(&mut self, _args: &[&str]) {
        self.send_message("Farewell! Have a great day!");
        std::process::exit(0);
    }
    fn history(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for command in &self.command_manager.command_buffer {
            results.push(command);
        }
        self.send_message(&format!("{:#?}", results));
        self.send_prompt();
    }
    fn program(&mut self, _args: &[&str]) {
        self.send_message("Listing instructions currently in VM's program vector:");
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(instruction.clone())
        }
        self.send_message(&format!("{:#?}", results));
        self.send_message(&format!("End of Program Listing"));
        self.send_prompt();
    }

    fn clear(&mut self, args: &[&str]) {
        if args.len() <= 0 {
            self.send_message("[Error]: Unknown argument to clear: program/registers");
            self.send_message("[Error]: For example: !clear program or !clear regiseters");
            self.send_prompt();
            return;
        }

        match args[0].to_lowercase().as_str() {
            "program" => {
                self.vm.program.clear();
            },
            "registers" => {
                self.vm.registers.iter_mut().for_each(|i| *i = 0);
            },
            "history" => {
                self.command_manager.clear_all();
            },
            _ => {
                self.send_message("[Error]: Unknown argument to clear: program/registers");
                self.send_message("[Error]: For example: !clear program or !clear regiseters");
            },
        }
        self.send_prompt();
    }

    fn symbols(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for symbol in &self.asm.symbols.symbols {
            results.push(symbol.clone());
        }
        self.send_message("Listing symbols table:");
        self.send_message(&format!("{:#?}", results));
        self.send_message("End of Symbols Listing");
        self.send_prompt();
    }
    fn registers(&mut self, _args: &[&str]) {
        self.send_message("Listing registers and all contents:");
        let mut results = vec![];
        for register in &self.vm.registers {
            results.push(register.clone());
        }
        self.send_message(&format!("{:#?}", results));
        self.send_message("End of Register Listing");
        self.send_prompt();
    }

    fn load_file(&mut self, _args: &[&str], data_from_file: &Option<String>) {
        if let Some(contents) = data_from_file {
            let program = match program(&contents) {
                Ok((_reminder, program)) => program,
                Err(e) => {
                    self.send_message(&format!("[Error]: Unable to parse input: {:?}", e));
                    self.send_prompt();
                    return;
                },
            };
            self.vm
                .program
                .append(&mut program.to_bytes(&self.asm.symbols));
        }
    }

    fn spawn(&mut self, _args: &[&str], data_from_file: &Option<String>) {
        if let Some(contents) = data_from_file {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    // println!("Sending assembled program to VM");
                    self.vm.program.append(&mut assembled_program);
                    // println!("{:#?}", self.vm.program);
                    self.scheduler.get_thread(self.vm.clone());
                },
                Err(errors) => {
                    for error in errors {
                        self.send_message(&format!("Unable to parse input: {:?}", error));
                        self.send_prompt();
                    }
                },
            }
        }
    }

    fn require_file_to_load(&mut self) -> Option<String> {
        let stdin = io::stdin();
        self.send_message("Please enter the path to the file you wish to load: ");
        let mut tmp = String::new();

        // io::stdout().flush().expect("Unable to flush stdout");

        stdin
            .read_line(&mut tmp)
            .expect("[Error]: Unable to read line from user");

        self.send_message("Attempting to load program from file...");

        utils::get_data_from_load(tmp)
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

mod utils {
    use std::io::Read;
    use std::path::PathBuf;

    use std::fs::File;

    use std::path::Path;

    pub fn get_data_from_load(tmp: String) -> Option<String> {
        // TODO: Change to the Result<O,E> return type so that msgs can be send to remote
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

    pub fn aggreate_path(args: &[&str]) -> Option<PathBuf> {
        if args.is_empty() {
            return None;
        }

        let mut left_single_quote = false;
        let mut left_double_quote = false;

        let mut path = PathBuf::new();

        for &arg in args {
            if !left_double_quote && !left_single_quote {
                if arg.starts_with("\"") && arg.ends_with("\"") {
                    path.push(&arg.trim_matches(&['\"']));
                    left_double_quote = false;
                    break;
                } else if arg.starts_with("\'") && arg.ends_with("\'") {
                    path.push(&arg.trim_matches(&['\'']));
                    left_single_quote = false;
                    break;
                } else if arg.starts_with("\"") {
                    left_double_quote = true;
                    path.push(&arg[1..]);
                } else if arg.starts_with("\'") {
                    left_single_quote = true;
                    path.push(&arg[1..]);
                }
            }

            if left_double_quote {
                if arg.ends_with("\"") {
                    path.push(&arg[..arg.len() - 1]);
                    left_double_quote = false;
                } else {
                    path.push(&arg);
                }
            } else if left_single_quote {
                if arg.ends_with("\'") {
                    path.push(&arg[..arg.len() - 1]);
                    left_double_quote = true;
                } else {
                    path.push(&arg);
                }
            }
        }

        if left_double_quote || left_single_quote {
            return None;
        }

        Some(path)
    }

    pub fn is_valid_path(path: &PathBuf) -> Option<String> {
        if path.has_root() {
            check_path_exists(&path)
        } else {
            let current_dir = match std::env::current_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    eprintln!("Unable to get current directory: {}", e);
                    return None;
                },
            };

            let abs_path = current_dir.join(path);
            check_path_exists(&abs_path)
        }
    }

    pub fn check_path_exists(path: &Path) -> Option<String> {
        if Path::exists(path) {
            match path.to_str() {
                Some(valid_str) => Some(String::from(valid_str)),
                None => {
                    eprintln!("Invalid UTF-8 in path: {:?}", path);
                    None
                },
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, path::PathBuf};

    use io::{Error, Read};

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
    fn test_load_file() {
        let test_file = get_absolute_path("docs/examples/counting_loop.iasm");

        let contents = match read_file_to_string(test_file.to_str().unwrap()) {
            Ok(content) => Some(content),
            Err(err) => panic!("Unable to read file:{}", err),
        };

        let mut repl = REPL::new();
        repl.load_file(&[""], &contents);
        assert!(repl.asm.errors.is_empty());

        let expect = vec![
            0, 0, 0, 100, 0, 1, 0, 1, 0, 2, 0, 0, 18, 2, 0, 0, 10, 0, 2, 0, 20, 0, 0, 0, 5, 0, 0, 0,
        ];
        assert_eq!(expect, repl.vm.program);
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

        let expect = vec![
            45, 50, 49, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0,
        ];

        assert_eq!(expect, repl.vm.program);
    }
}
