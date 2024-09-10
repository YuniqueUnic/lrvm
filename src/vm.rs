use crate::instruction::Opcode;

pub struct VM {
    // Simulate hard registers
    registers: [i32; 32], // Why we use array instead of vector? Because we know the size of registers at the start.
    // tracking the program counter
    pc: usize, // program counter, 8 bits
    // Running program bytes
    program: Vec<u8>, // program memory, 8 bits
    // The reminder of division operation
    reminder: u32,
    // the last compare result
    equal_flag: bool,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
            reminder: 0,
            equal_flag: false,
        }
    }

    pub fn run(&mut self) {
        let mut is_done = false;
        while !is_done {
            is_done = self.execute_instruction();
        }
    }

    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return false;
        }

        match self.decode_opcode() {
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize; // convert it to usize as the indexer of registers' array
                let number = self.next_16_bits(); // get the next 16 bits where store the number ready to store in the register
                self.registers[register] = number as i32; // store the number in the register
                                                          // continue;                                          // Start next iteration that waiting for reading the next 8 bits opcode
                false
            },
            Opcode::SUB => false,
            Opcode::MUL => false,
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
                false
            },
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.reminder = (register1 % register2) as u32;
                false
            },
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
                false
            },
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
                false
            },
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
                false
            },
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits(); //eat the next 8 bits
                false
            },
            Opcode::JMPE => {
                let register = self.next_8_bits() as usize;
                let target = self.registers[register];
                if self.equal_flag {
                    self.pc = target as usize;
                }
                false
            },
            Opcode::HLT => {
                println!("Hit the HLT");
                true
            },
            Opcode::IGL => {
                eprintln!("Illegal instruction encountered");
                true
            },
            _ => false,
        }
    }

    pub fn get_test_vm() -> VM {
        let mut test_vm = VM::new();
        test_vm.equal_flag = false;
        // test_vm.registers[0] = 5;
        // test_vm.registers[1] = 10;
        test_vm
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
        let result = ((self.program[self.pc] as u16) << 8) | (self.program[self.pc + 1] as u16);
        self.pc += 2;
        result
    }
}

/// The Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0);
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = VM::new();
        test_vm.program = vec![5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
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
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 4;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[1] = 6;
        test_vm.program = vec![0, 0, 0, 10, 8, 1, 0, 0, 0];
        test_vm.run_once(); // currently, the LOAD opcode has taken [0,0,0,10] => load 0 << 8 + 10 at the registers[0]
        assert_eq!(test_vm.pc, 4); // so the pc locate at the index 4 which is number 8;
        test_vm.run_once(); // start to decode the 8 to JMPB and then read the registers[1] = 6
        assert_eq!(test_vm.pc, 0); // due to current pc index is 6 so that it subtracts 6 = 0;
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = VM::get_test_vm();
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
        let mut test_vm = VM::get_test_vm();
        test_vm.equal_flag = true;
        test_vm.registers[0] = 7;
        test_vm.program = vec![15, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 7);
    }
}
