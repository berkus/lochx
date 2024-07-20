use {
    crate::{error::RuntimeError, opcode::OpCode, value::Value},
    culpa::throws,
    tabular::{Row, Table},
};

pub struct Chunk {
    code: Vec<u8>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn append_op(&mut self, op: OpCode) {
        op.write_to(&mut self.code);
    }

    pub fn append_const(&mut self, constant: Value) -> u8 {
        let pos = self.constants.len();
        self.constants.push(constant);
        pos as u8
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
        let (op_size, insn, details) = OpCode::try_from(&self.code, offset)?.disassemble(self);
        table.add_row(
            Row::new()
                .with_cell(offset)
                .with_cell(insn)
                .with_cell(details),
        );
        offset + op_size
    }
}
