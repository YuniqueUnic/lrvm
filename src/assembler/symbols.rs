#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    name: String,
    symbol_type: SymbolType,
    offset: Option<u32>,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset: None,
        }
    }

    pub fn new_with_offset(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset: Some(offset),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolType {
    Label,
    Integer,
    IrString,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { symbols: vec![] }
    }

    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    pub fn has_symbol(&self, s: &str) -> bool {
        self.symbols.iter().any(|symbol: &Symbol| symbol.name == s)
    }

    pub fn set_symbol_offset(&mut self, s: &str, offset: u32) -> bool {
        self.symbols.iter_mut().any(|symbol: &mut Symbol| {
            if symbol.name == s {
                symbol.offset = Some(offset);
                true
            } else {
                false
            }
        })

        // for symbol in &mut self.symbols {
        //     if symbol.name == s {
        //         symbol.offset = Some(offset);
        //         return true;
        //     }
        // }
        // false
    }

    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return symbol.offset;
            }
        }
        None
    }
}
