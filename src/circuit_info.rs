use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub input_name_to_wire_index: BTreeMap<String, usize>,
    pub constants: BTreeMap<String, ConstantInfo>,
    pub output_name_to_wire_index: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub value: String,
    pub wire_index: usize,
}
