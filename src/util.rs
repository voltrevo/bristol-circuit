use serde::{Deserialize, Serialize};
use strum_macros::{Display as StrumDisplay, EnumString};

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArithmeticGate {
    pub op: AGateType,
    pub lh_in: u32,
    pub rh_in: u32,
    pub out: u32,
}

/// The supported Arithmetic gate types.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, StrumDisplay)]
pub enum AGateType {
    AAdd,
    ADiv,
    AEq,
    AGEq,
    AGt,
    ALEq,
    ALt,
    AMul,
    ANeq,
    ASub,
    AXor,
    APow,
    AIntDiv,
    AMod,
    AShiftL,
    AShiftR,
    ABoolOr,
    ABoolAnd,
    ABitOr,
    ABitAnd,
}
