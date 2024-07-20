use {
    crate::{chunk::Chunk, error::RuntimeError, opcode::OpCode, value::Value},
    culpa::throws,
    tabular::Table,
    thiserror::Error,
};

pub struct VM<'vm> {
    chunk: &'vm Chunk,
    pc: usize, // *ptr to chunk->code
    stack: Vec<Value>,
}

#[derive(Error, Debug)]
pub enum InterpretError {
    #[error("Compile error")]
    CompileError,
    #[error("Runtime error: {0}")]
    RuntimeError(#[from] RuntimeError),
}

impl<'vm> VM<'vm> {
    pub fn new(chunk: &'vm Chunk, pc: usize) -> Self {
        Self {
            chunk,
            pc,
            stack: vec![],
        }
    }

    #[throws(InterpretError)]
    pub fn run(&mut self) {
        loop {
            for sv in &self.stack {
                print!("[{}]", sv);
            }
            println!();
            self.trace_insn()?;
            let insn = OpCode::try_from(&self.chunk.code, self.pc)?;
            self.pc += insn.size();
            match insn {
                OpCode::Return => {
                    println!("{:?}", self.stack.pop());
                    return;
                }
                OpCode::Constant(i) => self.stack.push(self.chunk.constants[i as usize]),
                _ => todo!(),
            }
        }
    }

    #[throws(RuntimeError)]
    fn trace_insn(&self) {
        let mut table = Table::new("{:<} {:>} {:<} {:<}");
        self.chunk.disassemble_instruction(&mut table, self.pc)?;
        println!("{}", table);
    }
}
