use crate::bristol_line::BristolLine;
use crate::gate::Gate;
use crate::raw_bristol_circuit::RawBristolCircuit;
use crate::{bristol_circuit_error::BristolCircuitError, circuit_info::CircuitInfo};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BristolCircuit {
    pub wire_count: usize,
    pub info: CircuitInfo,

    // None is used for arithmetic circuits, where all io is width 1
    // Some is used for boolean circuits and the widths indicate the number of bits in each input
    // and output
    pub io_widths: Option<(Vec<usize>, Vec<usize>)>,

    pub gates: Vec<Gate>,
}

impl BristolCircuit {
    pub fn from_raw(raw: &RawBristolCircuit) -> Result<BristolCircuit, BristolCircuitError> {
        BristolCircuit::from_info_and_bristol_string(&raw.info, &raw.bristol)
    }

    pub fn to_raw(&self) -> Result<RawBristolCircuit, BristolCircuitError> {
        Ok(RawBristolCircuit {
            bristol: self.get_bristol_string()?,
            info: self.info.clone(),
        })
    }

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
        info: &CircuitInfo,
        input: &str,
    ) -> Result<BristolCircuit, BristolCircuitError> {
        BristolCircuit::read_info_and_bristol(info, &mut BufReader::new(input.as_bytes()))
    }

    pub fn write_bristol<W: Write>(&self, w: &mut W) -> Result<(), BristolCircuitError> {
        writeln!(w, "{} {}", self.gates.len(), self.wire_count)?;

        if let Some((input_widths, output_widths)) = &self.io_widths {
            write!(w, "{}", input_widths.len())?;
            for width in input_widths {
                write!(w, " {}", width)?;
            }
            writeln!(w)?;

            write!(w, "{}", output_widths.len())?;
            for width in output_widths {
                write!(w, " {}", width)?;
            }
            writeln!(w)?;
        } else {
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
        }

        writeln!(w)?;

        for gate in &self.gates {
            writeln!(w, "{}", gate)?;
        }

        Ok(())
    }

    pub fn read_info_and_bristol<R: BufRead>(
        info: &CircuitInfo,
        r: &mut R,
    ) -> Result<BristolCircuit, BristolCircuitError> {
        let (gate_count, wire_count) = BristolLine::read(r)?.circuit_sizes()?;

        let input_widths = BristolLine::read(r)?.io_widths()?;
        if input_widths.len() != info.input_name_to_wire_index.len() {
            return Err(BristolCircuitError::Inconsistency {
                message: "Input count mismatch".into(),
            });
        }

        let output_widths = BristolLine::read(r)?.io_widths()?;
        if output_widths.len() != info.output_name_to_wire_index.len() {
            return Err(BristolCircuitError::Inconsistency {
                message: "Output count mismatch".into(),
            });
        }

        let io_widths = {
            let inputs_all_1 = input_widths.iter().all(|&x| x == 1);
            let outputs_all_1 = output_widths.iter().all(|&x| x == 1);

            if inputs_all_1 && outputs_all_1 {
                None
            } else {
                Some((input_widths, output_widths))
            }
        };

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
            info: info.clone(),
            io_widths,
            gates,
        })
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
            io_widths: None,
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
                &CircuitInfo {
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
        let io_widths = bristol_line.io_widths().unwrap();
        assert_eq!(io_widths, vec![1, 1]);
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
