use std::{io::BufRead, str::FromStr};

use crate::{bristol_circuit_error::BristolCircuitError, gate::Gate};

pub struct BristolLine(pub Vec<String>);

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

    pub fn get<T: FromStr>(&self, index: usize) -> Result<T, BristolCircuitError> {
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

    pub fn get_str(&self, index: usize) -> Result<&str, BristolCircuitError> {
        self.0
            .get(index)
            .ok_or(BristolCircuitError::ParsingError {
                message: format!("Index {} out of bounds", index),
            })
            .map(|s| s.as_str())
    }
}
