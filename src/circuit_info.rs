use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub input_name_to_wire_index: HashMap<String, usize>,
    pub constants: HashMap<String, ConstantInfo>,
    pub output_name_to_wire_index: HashMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub value: String,
    pub wire_index: usize,
}
