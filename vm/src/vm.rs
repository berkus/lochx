use {
    crate::{chunk::Chunk, error::RuntimeError, opcode::OpCode},
    culpa::throws,
    thiserror::Error,
};

pub struct VM<'vm> {
    chunk: &'vm Chunk,
    pc: usize, // *ptr to chunk->code
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
        Self { chunk, pc }
    }

    #[throws(InterpretError)]
    pub fn run(&mut self) {
        loop {
            let insn = OpCode::try_from(&self.chunk.code, self.pc)?;
            self.pc += insn.size();
            match insn {
                OpCode::Return => return,
                _ => todo!(),
            }
        }
    }
}
