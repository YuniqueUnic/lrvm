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
    IGL,     // _
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
}
