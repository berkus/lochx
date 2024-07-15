use {
    crate::error::RuntimeError,
    culpa::throws,
    std::fmt::Display,
    tabular::{Row, Table},
};

pub enum OpCode {
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = RuntimeError;

    #[throws(Self::Error)]
    fn try_from(value: u8) -> Self {
        match value {
            0 => OpCode::Return,
            _ => unimplemented!(),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OpCode::Return => "RETURN",
            }
        )
    }
}

pub struct Chunk {
    code: Vec<u8>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: vec![0] }
    }

    #[throws(RuntimeError)]
    pub fn disassemble(&self, title: impl AsRef<str>) {
        println!("=== {} ===", title.as_ref());
        //                          off  op   args
        let mut table = Table::new("{:<} {:<} {:<}");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(&mut table, offset)?;
        }
        println!("{}", table);
    }

    #[throws(RuntimeError)]
    fn disassemble_instruction(&self, table: &mut Table, offset: usize) -> usize {
        let insn: OpCode = self.code[offset].try_into()?;
        table.add_row(
            Row::new().with_cell(offset).with_cell(insn).with_cell(""), // .with_cell(insn.arg_decode()),
        );
        offset + 1
    }
}
