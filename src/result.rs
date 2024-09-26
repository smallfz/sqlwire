use {
    serde::Serialize,
    serde_json::Error as JSONError,
    std::{
        fmt,
        fmt::{Debug, Display},
    },
};

#[derive(Serialize, Debug, PartialEq)]
pub enum Error {
    JSON(String),
    Notfound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::JSON(v) => write!(f, "json: {}", &v),
            Self::Notfound(v) => write!(f, "parameter {} not found.", &v),
        }
    }
}

impl From<JSONError> for Error {
    fn from(e: JSONError) -> Self {
        Self::JSON(format!("{}", &e))
    }
}

pub type R = Result<(), Error>;
