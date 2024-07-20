use {
    crate::{error::RuntimeError, opcode::OpCode, value::Value},
    culpa::throws,
    tabular::{Row, Table},
};

pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    pub fn append_op(&mut self, op: OpCode, line: usize) {
        op.write_to(&mut self.code);
        for _ in 0..op.size() {
            self.lines.push(line);
        }
    }

    pub fn append_const(&mut self, constant: Value) -> u8 {
        let pos = self.constants.len();
        self.constants.push(constant);
        pos as u8
    }

    #[throws(RuntimeError)]
    pub fn disassemble(&self, title: impl AsRef<str>) {
        println!("=== {} ===", title.as_ref());
        //                          off  line op   args
        let mut table = Table::new("{:<} {:>} {:<} {:<}");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(&mut table, offset)?;
        }
        println!("{}", table);
    }

    #[throws(RuntimeError)]
    pub fn disassemble_instruction(&self, table: &mut Table, offset: usize) -> usize {
        let (op_size, insn, details) = OpCode::try_from(&self.code, offset)?.disassemble(self);
        let line = if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            "|".into()
        } else {
            format!("{}", self.lines[offset])
        };
        table.add_row(
            Row::new()
                .with_cell(offset)
                .with_cell(line)
                .with_cell(insn)
                .with_cell(details),
        );
        offset + op_size
    }
}
