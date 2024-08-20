use crate::bristol_circuit_error::BristolCircuitError;
use crate::gate::Gate;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BristolCircuit {
    pub wire_count: usize,
    pub info: CircuitInfo,
    pub gates: Vec<Gate>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub input_name_to_wire_index: HashMap<String, usize>,
    pub constants: HashMap<String, ConstantInfo>,
    pub output_name_to_wire_index: HashMap<String, usize>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub value: String,
    pub wire_index: usize,
}

impl BristolCircuit {
    pub fn get_bristol_string(&self) -> Result<String, BristolCircuitError> {
        let mut output = Vec::new();
        let mut writer = BufWriter::new(&mut output);

        self.write_bristol(&mut writer)?;
        drop(writer);

        String::from_utf8(output).map_err(|_| BristolCircuitError::ParsingError {
            message: "Generated Bristol data was not valid utf8".into(),
        })
    }

    pub fn from_info_and_bristol_string(
        info: CircuitInfo,
        input: &str,
    ) -> Result<BristolCircuit, BristolCircuitError> {
        BristolCircuit::read_info_and_bristol(info, &mut BufReader::new(input.as_bytes()))
    }

    pub fn write_bristol<W: Write>(&self, w: &mut W) -> Result<(), BristolCircuitError> {
        writeln!(w, "{} {}", self.gates.len(), self.wire_count)?;

        write!(w, "{}", self.info.input_name_to_wire_index.len())?;

        for _ in 0..self.info.input_name_to_wire_index.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;

        write!(w, "{}", self.info.output_name_to_wire_index.len())?;

        for _ in 0..self.info.output_name_to_wire_index.len() {
            write!(w, " 1")?;
        }

        writeln!(w)?;
        writeln!(w)?;

        for gate in &self.gates {
            writeln!(w, "{}", gate)?;
        }

        Ok(())
    }

    pub fn read_info_and_bristol<R: BufRead>(
        info: CircuitInfo,
        r: &mut R,
    ) -> Result<BristolCircuit, BristolCircuitError> {
        let (gate_count, wire_count) = BristolLine::read(r)?.circuit_sizes()?;

        let input_count = BristolLine::read(r)?.io_count()?;
        if input_count != info.input_name_to_wire_index.len() {
            return Err(BristolCircuitError::Inconsistency {
                message: "Input count mismatch".into(),
            });
        }

        let output_count = BristolLine::read(r)?.io_count()?;
        if output_count != info.output_name_to_wire_index.len() {
            return Err(BristolCircuitError::Inconsistency {
                message: "Output count mismatch".into(),
            });
        }

        let mut gates = Vec::new();
        for _ in 0..gate_count {
            gates.push(BristolLine::read(r)?.gate()?);
        }

        for line in r.lines() {
            if !line?.trim().is_empty() {
                return Err(BristolCircuitError::ParsingError {
                    message: "Unexpected non-whitespace line after gates".into(),
                });
            }
        }

        Ok(BristolCircuit {
            wire_count,
            info,
            gates,
        })
    }
}

struct BristolLine(Vec<String>);

impl BristolLine {
    pub fn read(r: &mut impl BufRead) -> Result<Self, BristolCircuitError> {
        loop {
            let mut line = String::new();
            r.read_line(&mut line)?;

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            return Ok(BristolLine(
                line.split_whitespace()
                    .map(|part| part.to_string())
                    .collect(),
            ));
        }
    }

    pub fn circuit_sizes(&self) -> Result<(usize, usize), BristolCircuitError> {
        Ok((self.get(0)?, self.get(1)?))
    }

    pub fn io_count(&self) -> Result<usize, BristolCircuitError> {
        let count = self.get::<usize>(0)?;

        if self.0.len() != (count + 1) {
            return Err(BristolCircuitError::ParsingError {
                message: format!("Expected {} parts", count + 1),
            });
        }

        for i in 1..self.0.len() {
            if self.get_str(i)? != "1" {
                return Err(BristolCircuitError::ParsingError {
                    message: format!("Expected 1 at index {}", i),
                });
            }
        }

        Ok(count)
    }

    pub fn gate(&self) -> Result<Gate, BristolCircuitError> {
        let input_len = self.get::<usize>(0)?;
        let output_len = self.get::<usize>(1)?;

        let expected_part_len = input_len + output_len + 3;

        if self.0.len() != expected_part_len {
            return Err(BristolCircuitError::ParsingError {
                message: format!(
                    "Inconsistent part length (actual: {}, expected: {})",
                    self.0.len(),
                    expected_part_len
                ),
            });
        }

        let mut inputs = Vec::<usize>::new();

        for i in 0..input_len {
            inputs.push(self.get(i + 2)?);
        }

        let mut outputs = Vec::<usize>::new();

        for i in 0..output_len {
            outputs.push(self.get(i + 2 + input_len)?);
        }

        let op = self.get::<String>(input_len + output_len + 2)?;

        Ok(Gate {
            inputs,
            outputs,
            op,
        })
    }

    fn get<T: FromStr>(&self, index: usize) -> Result<T, BristolCircuitError> {
        self.0
            .get(index)
            .ok_or(BristolCircuitError::ParsingError {
                message: format!("Index {} out of bounds", index),
            })?
            .parse::<T>()
            .map_err(|_| BristolCircuitError::ParsingError {
                message: format!("Failed to convert at index {}", index),
            })
    }

    fn get_str(&self, index: usize) -> Result<&str, BristolCircuitError> {
        self.0
            .get(index)
            .ok_or(BristolCircuitError::ParsingError {
                message: format!("Index {} out of bounds", index),
            })
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    // Helper function to create a sample BristolCircuit
    fn create_sample_circuit() -> BristolCircuit {
        BristolCircuit {
            // d = (a + b) * b
            // we need to use inputX and outputX to match deserialization from bristol format
            // which doesn't specify the wire names
            wire_count: 4,
            info: CircuitInfo {
                input_name_to_wire_index: [("input0".to_string(), 0), ("input1".to_string(), 1)]
                    .iter()
                    .cloned()
                    .collect(),
                constants: Default::default(),
                output_name_to_wire_index: [("output0".to_string(), 3)].iter().cloned().collect(),
            },
            gates: vec![
                Gate {
                    inputs: vec![0, 1],
                    outputs: vec![2],
                    op: "AAdd".to_string(),
                },
                Gate {
                    inputs: vec![2, 1],
                    outputs: vec![3],
                    op: "AMul".to_string(),
                },
            ],
        }
    }

    fn clean(src: &str) -> String {
        src.trim_start()
            .trim_end_matches(char::is_whitespace)
            .lines()
            .map(str::trim)
            .collect::<Vec<&str>>()
            .join("\n")
            + "\n"
    }

    #[test]
    fn test_write_bristol() {
        assert_eq!(
            create_sample_circuit().get_bristol_string().unwrap(),
            clean(
                "
                    2 4
                    2 1 1
                    1 1
                    
                    2 1 0 1 2 AAdd
                    2 1 2 1 3 AMul
                ",
            ),
        );
    }

    #[test]
    fn test_read_bristol() {
        assert_eq!(
            BristolCircuit::from_info_and_bristol_string(
                CircuitInfo {
                    input_name_to_wire_index: [
                        ("input0".to_string(), 0),
                        ("input1".to_string(), 1)
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    constants: Default::default(),
                    output_name_to_wire_index: [("output0".to_string(), 3)]
                        .iter()
                        .cloned()
                        .collect(),
                },
                "
                    2 4
                    2 1 1
                    1 1

                    2 1 0 1 2 AAdd
                    2 1 2 1 3 AMul
                "
            )
            .unwrap(),
            create_sample_circuit()
        );
    }

    #[test]
    fn test_bristol_line_read() {
        let input_data = "2 4\n";
        let mut reader = BufReader::new(Cursor::new(input_data));

        let bristol_line = BristolLine::read(&mut reader).unwrap();
        assert_eq!(bristol_line.0, vec!["2", "4"]);
    }

    #[test]
    fn test_bristol_line_circuit_sizes() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let (gate_count, wire_count) = bristol_line.circuit_sizes().unwrap();
        assert_eq!(gate_count, 2);
        assert_eq!(wire_count, 4);
    }

    #[test]
    fn test_bristol_line_io_count() {
        let bristol_line = BristolLine(vec!["2".to_string(), "1".to_string(), "1".to_string()]);
        let io_count = bristol_line.io_count().unwrap();
        assert_eq!(io_count, 2);
    }

    #[test]
    fn test_bristol_line_gate() {
        let bristol_line = BristolLine(vec![
            "2".to_string(),
            "1".to_string(),
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "AAdd".to_string(),
        ]);
        let gate = bristol_line.gate().unwrap();
        assert_eq!(gate.inputs, vec![0, 1]);
        assert_eq!(gate.outputs, vec![2]);
        assert_eq!(gate.op, "AAdd");
    }

    #[test]
    fn test_bristol_line_get() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let value: usize = bristol_line.get(0).unwrap();
        assert_eq!(value, 2);
    }

    #[test]
    fn test_bristol_line_get_str() {
        let bristol_line = BristolLine(vec!["2".to_string(), "4".to_string()]);
        let value = bristol_line.get_str(1).unwrap();
        assert_eq!(value, "4");
    }
}
