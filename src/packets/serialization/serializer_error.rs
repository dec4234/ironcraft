use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

pub enum SerializingErr {
    InvalidEndOfVarInt,
    VarTypeTooLong(String),
    CouldNotDeserializeString,
    InputEnded,
    LeftoverInput,
    UnknownFailure,
    UniqueFailure(String),
}

impl Debug for SerializingErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())?;
        Ok(())
    }
}

impl Display for SerializingErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializingErr::InvalidEndOfVarInt => {f.write_str("VarInt ended prematurely")},
            SerializingErr::VarTypeTooLong(s) => {
                f.write_str("The VarType did not end when it should have. ")?;
                f.write_str(s)
            },
            SerializingErr::UnknownFailure => {f.write_str("Unknown deserialization failure")},
            SerializingErr::CouldNotDeserializeString => {f.write_str("Could not deserialize String")},
            SerializingErr::InputEnded => {f.write_str("Input ended prematurely")},
            SerializingErr::UniqueFailure(s) => {f.write_str(s)},
            SerializingErr::LeftoverInput => {f.write_str("There is unused input data left")}
        }
    }
}

impl Error for SerializingErr {}

impl From<Utf8Error> for SerializingErr {
    fn from(value: Utf8Error) -> Self {
        Self::CouldNotDeserializeString
    }
}