use thiserror::Error;

#[derive(Error, Debug)]
pub enum CalcError {
    #[error("Division by zero")]
    DivByZero,
    #[error("Exponent cannot have a unit")]
    ExpByUnit,
    #[error("Different unit types")]
    DifferentUnitTypes,
    #[error("Cannot operate with units")]
    OperateWithUnits,
    #[error("Conversion error")]
    ConversionError,
    #[error("Missing unit")]
    MissingUnit,
    #[error(transparent)]
    RequestError(#[from] ureq::Error),
    #[error(transparent)]
    ReadlineError(#[from] rustyline::error::ReadlineError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DeError(#[from] quick_xml::DeError),
    #[error(transparent)]
    ErrorRef(#[from] &'static CalcError),
}
