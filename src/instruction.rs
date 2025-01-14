/// Represents an opcode, which tells our interpreter what to do with the following operands
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    LOAD,    // 0
    ADD,     // 1
    SUB,     // 2
    MUL,     // 3
    DIV,     // 4
    HLT,     // 5
    JMP,     // 6
    JMPF,    // 7
    JMPB,    // 8
    EQ,      // 9
    NEQ,     // 10
    GTE,     // 11
    LTE,     // 12
    LT,      // 13
    GT,      // 14
    JMPE,    // 15
    NOP,     // 16
    ALOC,    // 17
    INC,     // 18
    DEC,     // 19
    DJMPE,   // 20
    IGL,     // 100
    PRTS,    // 21
    LOADF64, // 22
    ADDF64,  // 23
    SUBF64,  // 24
    MULF64,  // 25
    DIVF64,  // 26
    EQF64,   // 27
    NEQF64,  // 28
    GTF64,   // 29
    GTEF64,  // 30
    LTF64,   // 31
    LTEF64,  // 32
    SHL,     // 33
    SHR,     // 34
    AND,     // 35
    OR,      // 36
    XOR,     // 37
    NOT,     // 38
    LUI,     // 39
    CLOOP,   // 40
    LOOP,    // 41
    LOADM,   // 42
    SETM,    // 43
    PUSH,    // 44
    POP,     // 45
    CALL,    // 46
    RET,     // 47
}

impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        match self {
            Opcode::LOAD => 0,
            Opcode::ADD => 1,
            Opcode::SUB => 2,
            Opcode::MUL => 3,
            Opcode::DIV => 4,
            Opcode::HLT => 5,
            Opcode::JMP => 6,
            Opcode::JMPF => 7,
            Opcode::JMPB => 8,
            Opcode::EQ => 9,
            Opcode::NEQ => 10,
            Opcode::GTE => 11,
            Opcode::LTE => 12,
            Opcode::LT => 13,
            Opcode::GT => 14,
            Opcode::JMPE => 15,
            Opcode::NOP => 16,
            Opcode::ALOC => 17,
            Opcode::INC => 18,
            Opcode::DEC => 19,
            Opcode::DJMPE => 20,
            Opcode::PRTS => 21,
            Opcode::LOADF64 => 22,
            Opcode::ADDF64 => 23,
            Opcode::SUBF64 => 24,
            Opcode::MULF64 => 25,
            Opcode::DIVF64 => 26,
            Opcode::EQF64 => 27,
            Opcode::NEQF64 => 28,
            Opcode::GTF64 => 29,
            Opcode::GTEF64 => 30,
            Opcode::LTF64 => 31,
            Opcode::LTEF64 => 32,
            Opcode::SHL => 33,
            Opcode::SHR => 34,
            Opcode::AND => 35,
            Opcode::OR => 36,
            Opcode::XOR => 37,
            Opcode::NOT => 38,
            Opcode::LUI => 39,
            Opcode::CLOOP => 40,
            Opcode::LOOP => 41,
            Opcode::LOADM => 42,
            Opcode::SETM => 43,
            Opcode::PUSH => 44,
            Opcode::POP => 45,
            Opcode::CALL => 46,
            Opcode::RET => 47,
            Opcode::IGL => 100,
        }
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        match value {
            0 => Opcode::LOAD,
            1 => Opcode::ADD,
            2 => Opcode::SUB,
            3 => Opcode::MUL,
            4 => Opcode::DIV,
            5 => Opcode::HLT,
            6 => Opcode::JMP,
            7 => Opcode::JMPF,
            8 => Opcode::JMPB,
            9 => Opcode::EQ,
            10 => Opcode::NEQ,
            11 => Opcode::GTE,
            12 => Opcode::LTE,
            13 => Opcode::LT,
            14 => Opcode::GT,
            15 => Opcode::JMPE,
            16 => Opcode::NOP,
            17 => Opcode::ALOC,
            18 => Opcode::INC,
            19 => Opcode::DEC,
            20 => Opcode::DJMPE,
            21 => Opcode::PRTS,
            22 => Opcode::LOADF64,
            23 => Opcode::ADDF64,
            24 => Opcode::SUBF64,
            25 => Opcode::MULF64,
            26 => Opcode::DIVF64,
            27 => Opcode::EQF64,
            28 => Opcode::NEQF64,
            29 => Opcode::GTF64,
            30 => Opcode::GTEF64,
            31 => Opcode::LTF64,
            32 => Opcode::LTEF64,
            33 => Opcode::SHL,
            34 => Opcode::SHR,
            35 => Opcode::AND,
            36 => Opcode::OR,
            37 => Opcode::XOR,
            38 => Opcode::NOT,
            39 => Opcode::LUI,
            40 => Opcode::CLOOP,
            41 => Opcode::LOOP,
            42 => Opcode::LOADM,
            43 => Opcode::SETM,
            44 => Opcode::PUSH,
            45 => Opcode::POP,
            46 => Opcode::CALL,
            47 => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

impl From<&str> for Opcode {
    fn from(value: &str) -> Self {
        match value {
            "load" => Opcode::LOAD,
            "add" => Opcode::ADD,
            "sub" => Opcode::SUB,
            "mul" => Opcode::MUL,
            "div" => Opcode::DIV,
            "hlt" => Opcode::HLT,
            "jmp" => Opcode::JMP,
            "jmpf" => Opcode::JMPF,
            "jmpb" => Opcode::JMPB,
            "eq" => Opcode::EQ,
            "neq" => Opcode::NEQ,
            "gte" => Opcode::GTE,
            "lte" => Opcode::LTE,
            "lt" => Opcode::LT,
            "gt" => Opcode::GT,
            "jmpe" => Opcode::JMPE,
            "nop" => Opcode::NOP,
            "aloc" => Opcode::ALOC,
            "inc" => Opcode::INC,
            "dec" => Opcode::DEC,
            "djmpe" => Opcode::DJMPE,
            "igl" => Opcode::IGL,
            "prts" => Opcode::PRTS,
            "loadf64" => Opcode::LOADF64,
            "addf64" => Opcode::ADDF64,
            "subf64" => Opcode::SUBF64,
            "mulf64" => Opcode::MULF64,
            "divf64" => Opcode::DIVF64,
            "eqf64" => Opcode::EQF64,
            "neqf64" => Opcode::NEQF64,
            "gtf64" => Opcode::GTF64,
            "gtef64" => Opcode::GTEF64,
            "ltf64" => Opcode::LTF64,
            "ltef64" => Opcode::LTEF64,
            "shl" => Opcode::SHL,
            "shr" => Opcode::SHR,
            "and" => Opcode::AND,
            "or" => Opcode::OR,
            "xor" => Opcode::XOR,
            "not" => Opcode::NOT,
            "lui" => Opcode::LUI,
            "cloop" => Opcode::CLOOP,
            "loop" => Opcode::LOOP,
            "loadm" => Opcode::LOADM,
            "setm" => Opcode::SETM,
            "push" => Opcode::PUSH,
            "pop" => Opcode::POP,
            "call" => Opcode::CALL,
            "ret" => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

#[allow(dead_code)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction { opcode }
    }
}

/// The Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn test_create_igl() {
        let opcode = Opcode::IGL;
        assert_eq!(opcode, Opcode::IGL);
    }

    #[test]
    fn test_str_to_opcode() {
        let opcode = Opcode::from("hlt");
        assert_eq!(opcode, Opcode::HLT);
        let opcode = Opcode::from("illegal");
        assert_eq!(opcode, Opcode::IGL);
    }
}
