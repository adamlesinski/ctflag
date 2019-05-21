use std::env;
use std::fmt;
use std::str::FromStr;

pub use ctflag_derive::*;

#[derive(Debug, Clone)]
pub enum FlagError {
    ParseError(ParseErrorStruct),
    MissingValue(String),
    UnrecognizedArg(String),
}

#[derive(Clone, Debug)]
pub struct ParseErrorStruct {
    pub type_str: &'static str,
    pub input: String,
    pub src: FromArgError,
}

pub type Result<T> = std::result::Result<T, FlagError>;

pub trait Flags: Sized {
    fn from_args(args: env::Args) -> Result<(Self, Vec<String>)>;
    fn description() -> String;
}

#[derive(Clone, Debug)]
pub struct FromArgError {
    msg: Option<String>,
}

pub type FromArgResult<T> = std::result::Result<T, FromArgError>;

pub trait FromArg: Sized {
    fn from_arg(value: &str) -> FromArgResult<Self>;
}

pub fn bool_from_arg(s: Option<&str>) -> FromArgResult<bool> {
    match s {
        Some(s) => s.parse::<bool>().map_err(|_| FromArgError::new()),
        None => Ok(true),
    }
}

pub fn option_from_arg<T: FromArg>(s: &str) -> FromArgResult<Option<T>> {
    <T as FromArg>::from_arg(s).map(Some)
}

impl<T> FromArg for T
where
    T: FromStr,
{
    fn from_arg(s: &str) -> FromArgResult<T> {
        <T as FromStr>::from_str(s).map_err(|_| FromArgError::new())
    }
}

impl fmt::Display for FlagError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FlagError::ParseError(err) => {
                write!(
                    f,
                    "failed to parse \"{}\" as {} type",
                    &err.input, err.type_str
                )?;
                if let Some(msg) = &err.src.msg {
                    write!(f, ": {}", msg)?;
                }
            }
            FlagError::MissingValue(arg) => {
                write!(f, "missing value for argument \"{}\"", arg)?;
            }
            FlagError::UnrecognizedArg(arg) => {
                write!(f, "unrecognized argument \"{}\"", arg)?;
            }
        }
        Ok(())
    }
}

impl FromArgError {
    pub fn new() -> Self {
        FromArgError { msg: None }
    }

    pub fn with_message<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        FromArgError {
            msg: Some(format!("{}", msg)),
        }
    }
}
