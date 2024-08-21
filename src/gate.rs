use core::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gate {
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    pub op: String,
}

impl Display for Gate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inputs.len())?;
        write!(f, " {}", self.outputs.len())?;

        for i in &self.inputs {
            write!(f, " {}", i)?;
        }

        for o in &self.outputs {
            write!(f, " {}", o)?;
        }

        write!(f, " {}", self.op)
    }
}
