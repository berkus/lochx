use {
    crate::{chunk::Chunk, error::RuntimeError},
    culpa::{throw, throws},
};

pub enum OpCode {
    Return,
    Constant(u8),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl OpCode {
    pub fn disassemble(&self, chunk: &Chunk) -> (usize, &'static str, String) {
        match self {
            OpCode::Return => (1, "RETURN", "".into()),
            OpCode::Constant(i) => (
                2,
                "CONSTANT",
                format!("{} (={})", i, chunk.constants[*i as usize]),
            ),
            OpCode::Negate => (1, "NEGATE", "".into()),
            OpCode::Add => (1, "ADD", "".into()),
            OpCode::Subtract => (1, "SUBTRACT", "".into()),
            OpCode::Multiply => (1, "MULTIPLY", "".into()),
            OpCode::Divide => (1, "DIVIDE", "".into()),
        }
    }

    pub fn write_to(&self, place: &mut Vec<u8>) {
        match self {
            OpCode::Return => {
                place.push(0);
            }
            OpCode::Constant(i) => {
                place.push(1);
                place.push(*i);
            }
            OpCode::Negate => {
                place.push(2);
            }
            OpCode::Add => {
                place.push(3);
            }
            OpCode::Subtract => {
                place.push(4);
            }
            OpCode::Multiply => {
                place.push(5);
            }
            OpCode::Divide => {
                place.push(6);
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            OpCode::Return => 1,
            OpCode::Constant(_) => 2,
            OpCode::Negate => 1,
            OpCode::Add => 1,
            OpCode::Subtract => 1,
            OpCode::Multiply => 1,
            OpCode::Divide => 1,
        }
    }

    #[throws(RuntimeError)]
    pub fn try_from(place: &[u8], offset: usize) -> OpCode {
        match place[offset] {
            0 => OpCode::Return,
            1 => OpCode::Constant(place[offset + 1]),
            2 => OpCode::Negate,
            3 => OpCode::Add,
            4 => OpCode::Subtract,
            5 => OpCode::Multiply,
            6 => OpCode::Divide,
            x => throw!(RuntimeError::UnknownOpcode(x)),
        }
    }
}
