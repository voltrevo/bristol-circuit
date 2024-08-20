use serde::{Deserialize, Serialize};

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gate {
    pub op: String,
    pub lh_in: u32,
    pub rh_in: u32,
    pub out: u32,
}
