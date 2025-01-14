use std::{
    f64::EPSILON,
    io::Cursor,
    net::SocketAddr,
    sync::{Arc, RwLock},
    thread,
};

use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use uuid::Uuid;

use crate::{
    assembler::PIE_HEADER_PREFIX,
    cluster::{self, manager::Manager},
    instruction::Opcode,
    util::display,
};

pub const DEFAULT_HEAP_STARTING_SIZE: usize = 64;

pub fn get_test_vm() -> VM {
    let mut test_vm = VM::new();
    test_vm.equal_flag = false;
    test_vm.registers[0] = 5;
    test_vm.registers[1] = 10;
    test_vm.float_registers[0] = 5.0;
    test_vm.float_registers[1] = 10.0;
    test_vm
}

#[derive(Debug, Clone)]
enum VMEventType {
    Start,
    GracefulStop { code: u32 },
    Crash { code: u32 },
}

#[derive(Debug, Clone)]
pub struct VMEvent {
    event: VMEventType,
    at: DateTime<Utc>,
    application_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct VM {
    // Simulate hard registers
    pub registers: [i32; 32], // Why we use array instead of vector? Because we know the size of registers at the start.
    /// Array that simulates having floating point hardware registers
    pub float_registers: [f64; 32],
    // Running program bytes
    pub program: Vec<u8>, // program memory, 8 bits
    /// Number of logical cores the system reports
    pub logical_cores: usize,
    /// An alias that can be specified by the user and used to refer to the Node
    pub alias: Option<String>,
    /// Data structure to manage remote clients
    pub connection_manager: Arc<RwLock<Manager>>,
    // tracking the program counter
    pc: usize, // program counter, 8 bits
    // the heap memory
    heap: Vec<u8>, // heap memory, 8 bits
    /// Used to represent the stack
    stack: Vec<u8>,
    // The reminder of division operation
    reminder: usize,
    // the last compare result
    equal_flag: bool,
    /// Loop counter field, used with the `LOOP` instruction
    loop_counter: usize,
    /// Contains the read-only section data
    ro_data: Vec<u8>,
    /// 用于标识这个虚拟机的唯一随机生成的 UUID
    pub id: Uuid,
    /// Keeps a list of events for a particular VM
    events: Vec<VMEvent>,
    /// Server address that the VM will bind to for server-to-server communications
    server_addr: Option<String>,
    /// Port the server will bind to for server-to-server communications
    pub server_port: Option<String>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            float_registers: [0.0; 32],
            program: vec![],
            ro_data: vec![],
            heap: vec![0; DEFAULT_HEAP_STARTING_SIZE],
            stack: vec![],
            connection_manager: Arc::new(RwLock::new(Manager::new())),
            pc: 0,
            loop_counter: 0,
            reminder: 0,
            equal_flag: false,
            id: Uuid::new_v4(),
            alias: None,
            events: Vec::new(),
            logical_cores: num_cpus::get(),
            server_addr: None,
            server_port: None,
        }
    }

    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events.push(VMEvent {
            event: VMEventType::Start,
            at: Utc::now(),
            application_id: self.id.clone(),
        });

        if !self.verify_header() {
            self.events.push(VMEvent {
                event: VMEventType::Crash { code: 1 },
                at: Utc::now(),
                application_id: self.id.clone(),
            });
            display::writeout("Header was incorrect");
            return self.events.clone();
        }
        // If the header is valid, we need to change the PC to be at bit 65.
        self.pc = 64 + 4 + self.get_starting_offset();

        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }

        self.events.push(VMEvent {
            event: VMEventType::GracefulStop {
                code: is_done.unwrap(),
            },
            at: Utc::now(),
            application_id: self.id.clone(),
        });
        self.events.clone()
    }

    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn verify_header(&self) -> bool {
        self.program[0..4] == PIE_HEADER_PREFIX
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    pub fn add_bytes(&mut self, mut bytes: Vec<u8>) {
        self.program.append(&mut bytes);
    }

    fn execute_instruction(&mut self) -> Option<u32> {
        if self.pc >= self.program.len() {
            return Some(1);
        }

        match self.decode_opcode() {
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize; // convert it to usize as the indexer of registers' array
                let number = self.next_16_bits(); // get the next 16 bits where store the number ready to store in the register
                self.registers[register] = number as i32; // store the number in the register
                                                          // continue;                                          // Start next iteration that waiting for reading the next 8 bits opcode
            },
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            },
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            },
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                debug!("register1:{:?}, register2:{:?}", register1, register2);
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            },
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.reminder = (register1 % register2) as usize;
            },
            Opcode::HLT => {
                info!("Hit the HLT");
                return Some(0);
            },
            Opcode::IGL => {
                display::e_writeout("Illegal instruction encountered");
                return Some(1);
            },
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            },
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            },
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            },
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::GTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::LTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits(); //eat the next 8 bits
            },
            Opcode::JMPE => {
                let register = self.next_8_bits() as usize;
                let target = self.registers[register];
                if self.equal_flag {
                    self.pc = target as usize;
                } else {
                    // TODO: Fix the bits
                }
            },
            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            },
            Opcode::PRTS => {
                // PRTS 需要一个操作数，要么是字节码的只读部分中的起始索引
                // 或者是一个符号（以 @symbol_name 的形式），它将在符号表中查找偏移量。
                // 这条指令然后读取每个字节并打印它，直到它遇到一个 0x00 字节，这表示字符串的终止
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();

                // TODO: 是否能够找到一个更好的方法来做这个。也许我们可以存储字节长度而不是空终止？
                // 或者某种形式的缓存，我们在 VM 启动时就通过整个 ro_data 并找到每个字符串及其结束字节位置？
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);

                match result {
                    Ok(s) => {
                        print!("{}", s);
                    },
                    Err(e) => {
                        error!("为 prts 指令解码字符串时出错：{:#?}", e)
                    },
                }
            },
            Opcode::LOADF64 => {
                let register = self.next_8_bits() as usize;
                let num = f64::from(self.next_16_bits());
                self.float_registers[register] = num;
            },
            Opcode::ADDF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 + register2;
            },
            Opcode::SUBF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 - register2;
            },
            Opcode::MULF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 * register2;
            },
            Opcode::DIVF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 / register2;
                self.reminder = (register1 % register2) as usize;
            },
            Opcode::EQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() < EPSILON;
                self.next_8_bits();
            },
            Opcode::NEQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = !((register1 - register2).abs() < EPSILON);
                self.next_8_bits();
            },
            Opcode::GTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() > EPSILON && register1 > register2;
                self.next_8_bits();
            },
            Opcode::GTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag =
                    (register1 - register2).abs() >= EPSILON && register1 >= register2;
                self.next_8_bits();
            },
            Opcode::LTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() > EPSILON && register1 < register2;
                self.next_8_bits();
            },
            Opcode::LTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag =
                    (register1 - register2).abs() >= EPSILON && register1 <= register2;
                self.next_8_bits();
            },
            Opcode::SHL => {
                let reg_num = self.next_8_bits() as usize; // Gets the register the user wants to shift
                                                           // Gets the next 8 bits, which is how many bits they want to shift
                let num_bits = match self.next_8_bits() {
                    0 => 16,        // If it is 0, it defaults to 16 bits
                    other => other, // If it is some other number, it shifts that amount
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shl(num_bits.into());
            },
            Opcode::SHR => {
                let reg_num = self.next_8_bits() as usize; // Gets the register the user wants to shift
                                                           // Gets the next 8 bits, which is how many bits they want to shift
                let num_bits = match self.next_8_bits() {
                    0 => 16,        // If it is 0, it defaults to 16 bits
                    other => other, // If it is some other number, it shifts that amount
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shr(num_bits.into());
            },
            Opcode::AND => {},
            _ => display::e_writeout(&format!(
                "Unknown opcode:{:?} has not been impl;",
                self.decode_opcode()
            )),
        }
        None
    }

    fn get_starting_offset(&self) -> usize {
        // We only want to read the slice containing the 4 bytes right after the magic number
        let mut rdr = Cursor::new(&self.program[64..68]);
        // Read it as a u32, cast as a usize (since the VM's PC attribute is a usize), and return it
        rdr.read_u32::<LittleEndian>().unwrap() as usize
    }

    pub fn with_alias(mut self, alias: String) -> Self {
        if alias.is_empty() {
            self.alias = None;
        } else {
            self.alias = Some(alias);
        }
        self
    }

    pub fn with_cluster_bind(mut self, server_addr: String, server_port: String) -> Self {
        display::writeout(&format!("Binding VM to {}:{}", server_addr, server_port));
        self.server_addr = Some(server_addr);
        self.server_port = Some(server_port);
        self
    }
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    // Attempts to decode the next byte into an opcode
    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    // Grabs the next 16 bits (2 bytes)
    fn next_16_bits(&mut self) -> u16 {
        let result = (u16::from(self.program[self.pc]) << 8) | u16::from(self.program[self.pc + 1]);
        self.pc += 2;
        result
    }

    pub fn bind_cluster_server(&mut self) {
        if let Some(ref addr) = self.server_addr {
            if let Some(ref port) = self.server_port {
                display::writeout(&format!("Binding to: {} {}", addr, port));
                let socket_addr: SocketAddr = (addr.to_string() + ":" + port).parse().unwrap();
                display::writeout(&format!("SocketAddr is: {:?}", socket_addr));

                let clone_manager = self.connection_manager.clone();
                thread::spawn(move || {
                    cluster::server::listen(socket_addr, clone_manager);
                });
            } else {
                display::e_writeout(&format!(
                    "Unable to bind to cluster server address: {}",
                    addr
                ));
            }
        } else {
            display::e_writeout(&format!(
                "Unable to bind to cluster server port: {:?}",
                self.server_port
            ));
        }
    }
}

/// The Tests
#[cfg(test)]
mod tests {
    use std::vec;

    use log::debug;

    use crate::{
        assembler::prepend_header,
        vm::{get_test_vm, DEFAULT_HEAP_STARTING_SIZE},
    };

    use super::VM;

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0);
    }

    #[test]
    fn test_hlt_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_igl_opcode() {
        let mut test_vm = VM::new();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![0, 0, 1, 244]; // Remember, this is how we represent 500 using two u8 in little endian format
                                              // [0, 0, 1, 244] => next_16_bits() return the 0x100_000_000 + 244 = 256 + 244 = 500
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![1, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 15);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![2, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 5);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![3, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        debug!(
            "\test_vm.program:{:?} ---> len:{:?}\n",
            test_vm.program,
            test_vm.program.len()
        );
        test_vm.run();
        assert_eq!(
            test_vm.registers[2], 50,
            "test_vm.registers:{:?}",
            test_vm.registers
        );
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![4, 1, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 2);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 4;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 6;
        test_vm.program = vec![0, 0, 0, 10, 8, 1, 0, 0, 0];
        test_vm.run_once(); // currently, the LOAD opcode has taken [0,0,0,10] => load 0 << 8 + 10 at the registers[0]
        assert_eq!(test_vm.pc, 4); // so the pc locate at the index 4 which is number 8;
        test_vm.run_once(); // start to decode the 8 to JMPB and then read the registers[1] = 6
        assert_eq!(test_vm.pc, 0); // due to current pc index is 6 so that it subtracts 6 = 0;
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 6;
        test_vm.registers[1] = 6;
        test_vm.program = vec![9, 0, 1, 10, 9, 1, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 4);
        test_vm.registers[0] = 0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        assert_eq!(test_vm.pc, 8);
    }

    #[test]
    fn test_jmpe_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.equal_flag = true;
        test_vm.registers[0] = 7;
        test_vm.program = vec![15, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_neq_opcdoe() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![10, 1, 0, 22, 10, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 1;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_gte_opcdoe() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 12;
        test_vm.registers[0] = 10;
        test_vm.program = vec![11, 1, 0, 22, 11, 1, 0, 0, 11, 1, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 1;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lte_opcdoe() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 10;
        test_vm.registers[0] = 12;
        test_vm.program = vec![12, 1, 0, 22, 12, 1, 0, 0, 12, 1, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 13;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lt_opcdoe() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 10;
        test_vm.registers[0] = 12;
        test_vm.program = vec![13, 1, 0, 22, 13, 1, 0, 0, 13, 1, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[1] = 13;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gt_opcdoe() {
        let mut test_vm = get_test_vm();
        test_vm.registers[1] = 12;
        test_vm.registers[0] = 10;
        test_vm.program = vec![14, 1, 0, 22, 14, 1, 0, 0, 14, 1, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[1] = 1;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024 + DEFAULT_HEAP_STARTING_SIZE);
        // the end size of heap should be the default starting size + new allocated size
    }

    #[test]
    fn test_prts_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]); // "Hello"
        test_vm.program = vec![21, 0, 0, 0];
        test_vm.run_once();
        // TODO: How can we validate the output since it is just printing to stdout in a test?
    }

    #[test]
    fn test_shl_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![33, 0, 0, 0];
        assert_eq!(5, test_vm.registers[0]);
        test_vm.run_once();
        assert_eq!(327680, test_vm.registers[0]);
    }
}
