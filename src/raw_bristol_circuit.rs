use serde::{Deserialize, Serialize};

use crate::circuit_info::CircuitInfo;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawBristolCircuit {
    pub bristol: String,
    pub info: CircuitInfo,
}
