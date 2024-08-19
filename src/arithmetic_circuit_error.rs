use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArithmeticCircuitError {
    #[error("Parsing error: {message}")]
    ParsingError { message: String },
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Inconsistency: {message}")]
    Inconsistency { message: String },
}
