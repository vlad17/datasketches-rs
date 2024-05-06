use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DataSketchesError {
    CXXError(String),
    DecodeError(String),
}

impl Display for DataSketchesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSketchesError::CXXError(err) => f.write_fmt(format_args!("Error: {}", err)),
            DataSketchesError::DecodeError(err) => {
                f.write_fmt(format_args!("DecodeError: {}", err))
            }
        }
    }
}

impl std::error::Error for DataSketchesError {}

impl From<base64::DecodeError> for DataSketchesError {
    fn from(value: base64::DecodeError) -> Self {
        Self::DecodeError(format!("{}", value))
    }
}

impl From<cxx::Exception> for DataSketchesError {
    fn from(value: cxx::Exception) -> Self {
        Self::CXXError(format!("{}", value))
    }
}
