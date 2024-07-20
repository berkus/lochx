use {
    crate::{chunk::Chunk, error::RuntimeError},
    culpa::{throw, throws},
};

pub enum OpCode {
    Return,
    Constant(u8),
}

impl OpCode {
    pub fn disassemble(&self, chunk: &Chunk) -> (usize, String, String) {
        match self {
            OpCode::Return => (1, "RETURN".into(), "".into()),
            OpCode::Constant(i) => (
                2,
                "CONSTANT".into(),
                format!("{} (={})", i, chunk.constants[*i as usize]),
            ),
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
        }
    }

    #[throws(RuntimeError)]
    pub fn try_from(place: &[u8], offset: usize) -> OpCode {
        match place[offset] {
            0 => OpCode::Return,
            1 => OpCode::Constant(place[offset + 1]),
            x => throw!(RuntimeError::UnknownOpcode(x)),
        }
    }
}

// impl Display for OpCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 OpCode::Return => "RETURN",
//             }
//         )
//     }
// }
