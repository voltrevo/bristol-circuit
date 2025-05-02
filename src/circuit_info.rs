use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub constants: Vec<ConstantInfo>,
    pub inputs: Vec<IOInfo>,
    pub outputs: Vec<IOInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: Value,

    pub address: usize, // aka wire_index
    pub width: usize,

    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IOInfo {
    pub name: String,

    #[serde(rename = "type")]
    pub type_: Value,

    pub address: usize, // aka wire_index
    pub width: usize,
}

impl CircuitInfo {
    pub fn input_widths(&self) -> Vec<usize> {
        self.inputs.iter().map(|input| input.width).collect()
    }

    pub fn output_widths(&self) -> Vec<usize> {
        self.outputs.iter().map(|output| output.width).collect()
    }
}
