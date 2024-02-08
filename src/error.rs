#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Constant Pool {0}")]
    ConstantPoolError(String),
    #[error("Binary Parsing Error")]
    BinrwError(#[from] binrw::Error),
    #[error("Text Processing Error")]
    NomError(nom::Err<nom::error::Error<String>>),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("InvokeDynamic instruction found without a class BootstrapMethods attribute")]
    NoBootstrapMethods,
    #[error("Invalid bootstrap method index `{0}`.")]
    InvalidBootstrapIndex(u16), 
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match value {
            nom::Err::Incomplete(x) => Self::NomError(nom::Err::Incomplete(x)),
            nom::Err::Error(x) => Self::NomError(nom::Err::Error(nom::error::Error::new(
                x.input.to_string(),
                x.code,
            ))),
            nom::Err::Failure(x) => Self::NomError(nom::Err::Failure(nom::error::Error::new(
                x.input.to_string(),
                x.code,
            ))),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;