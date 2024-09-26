use nom::{
    branch::alt,
    character::complete::{line_ending, multispace0},
    combinator::{eof, map, opt},
    error::context,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use super::{
    label_parsers::label_declaration, opcode_parsers::opcode, operand_parser::operand, SymbolTable,
    Token,
};

#[derive(Debug, PartialEq, Clone)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// 将指令转换为字节码
    ///
    /// 本函数负责将 CPU 指令转换为字节码表示。它首先处理指令的操作码部分，
    /// 然后依次处理指令的每个操作数。如果操作码或操作数不能正确转换为字节码，
    /// 函数将打印错误信息并终止程序。
    ///
    /// 返回：
    ///     一个包含字节码的向量，表示该 CPU 指令
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        // 初始化存储字节码的向量
        let mut results: Vec<u8> = vec![];

        // 根据操作码将其转换为字节码
        match self.opcode {
            Some(Token::Op { code }) => {
                // 将操作码转换为 u8 类型并添加到结果向量中
                results.push(code.into());
            },
            _ => {
                // 如果操作码字段中没有操作码，打印信息并终止程序
                eprintln!("Non-Opcode found in opcode field: {:?}", self.opcode);
                // std::process::exit(1);
            },
        }

        // 遍历指令的操作数，将它们转换为字节码
        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                // 如果操作数存在，调用提取函数将其添加到结果向量中
                AssemblerInstruction::extract_operand(token, &mut results, symbols);
            }
        }

        while 0 < results.len() && results.len() < 4 {
            results.push(0);
        }

        // 返回包含指令字节码的向量
        results
    }

    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }

    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    /// Checks if the AssemblyInstruction has any operands at all
    pub fn has_operands(&self) -> bool {
        self.operand1.is_some() || self.operand2.is_some() || self.operand3.is_some()
    }

    pub fn get_label_name(&self) -> Option<String> {
        match &self.label {
            Some(Token::LabelDeclaration { name }) => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn get_directive_name(&self) -> Option<String> {
        match &self.directive {
            Some(Token::Directive { name }) => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn get_string_constant(&self) -> Option<String> {
        match &self.operand1 {
            Some(Token::IrString { name }) => Some(name.to_string()),
            _ => None,
        }
    }

    /// 从解析令牌中提取操作数并将其转换为字节后存储到结果向量中。
    ///
    /// 该函数根据传入的令牌类型执行不同的操作以提取操作数。
    /// - 对于寄存器类型的令牌，它将寄存器编号作为单个字节提取。
    /// - 对于整数操作数类型的令牌，它将操作数值转换为两个字节后提取。
    /// - 对于其他类型的令牌，它打印错误信息并退出程序。
    ///
    /// 参数：
    /// - t: 指向包含操作数信息的令牌的引用。
    /// - results: 操作数提取后将字节数据推入此向量。
    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            // 对于寄存器类型的令牌，提取并存储寄存器编号。
            Token::Register { reg_num } => {
                results.push(*reg_num);
            },
            // 对于整数操作数类型的令牌，将其值转换为两个字节后提取并存储。
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                // 利用大端序规则，将最高有效字节首先存储。
                // obuse the big endian rule that store the most significant byte first
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            },
            Token::LabelUsage { name } => {
                if let Some(value) = symbols.symbol_value(name) {
                    let byte1 = value;
                    let byte2 = value >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                }
            },
            // 对于其他所有令牌类型，打印错误信息并退出程序。
            _ => {
                println!("Opcode found in operand field: {:#?}", t);
                // std::process::exit(1);
            },
        }
    }
}

fn instruction_combined(input: &str) -> IResult<&str, AssemblerInstruction> {
    context(
        // use context to show better error msg when failed to parse
        "instruction_combined",
        preceded(
            multispace0,
            terminated(
                map(
                    tuple((
                        opt(label_declaration),
                        opcode,
                        opt(operand),
                        opt(operand),
                        opt(operand),
                    )),
                    |(l, o, o1, o2, o3)| AssemblerInstruction {
                        opcode: Some(o),
                        label: l,
                        directive: None,
                        operand1: o1,
                        operand2: o2,
                        operand3: o3,
                    },
                ),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
}

pub fn instruction(input: &str) -> IResult<&str, AssemblerInstruction> {
    context("instruction", alt((instruction_combined,)))(input)
}

#[cfg(test)]
mod tests {
    use crate::{assembler::Token, instruction::Opcode};

    use super::{instruction_combined, AssemblerInstruction};

    #[test]
    fn test_parse_instruction_form_one() {
        let expect = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::LOAD }),
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: Some(Token::IntegerOperand { value: 100 }),
            operand3: None,
            label: None,
            directive: None,
        };

        let result = instruction_combined("load $0 #100\n");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_combined("  load $0 #100     \n    ");
        assert_eq!(result, Ok(("", expect)));
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let expect = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::HLT }),
            operand1: None,
            operand2: None,
            operand3: None,
            label: None,
            directive: None,
        };

        let result = instruction_combined("hlt\n");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_combined("hlt \n    ");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_combined("     hlt \n    ");
        assert_eq!(result, Ok(("", expect)));
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let expect = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::ADD }),
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: Some(Token::Register { reg_num: 1 }),
            operand3: Some(Token::Register { reg_num: 2 }),
            label: None,
            directive: None,
        };

        let result = instruction_combined("add $0 $1 $2\n");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_combined("  add    $0 $1    $2\n");
        assert_eq!(result, Ok(("", expect)));
    }

    #[test]
    fn test_parse_instruction_form_four() {
        let expect = AssemblerInstruction {
            opcode: Some(Token::Op { code: Opcode::INC }),
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: None,
            operand3: None,
            label: Some(Token::LabelDeclaration {
                name: String::from("test"),
            }),
            directive: None,
        };

        let result = instruction_combined("test: inc $0\n");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_combined("  test: inc $0 \n    ");
        assert_eq!(result, Ok(("", expect.clone())));
    }
}
