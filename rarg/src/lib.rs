use std::env;

pub use rarg_derive::*;

#[derive(Debug, Clone)]
pub enum ParseError {
    FlagError(String),
}

pub trait Flags: Sized {
    fn from_args(args: env::Args) -> Result<Self, ParseError>;
    fn description() -> String;
}
