#[derive(Debug, PartialEq)]
pub enum VmmError {
    NotSupported,
    NoFound,
    Unimplemented,
    TranslationError,
    DeviceNotFound,
    PseudoInst,
    DecodeInstError,
    UnexpectedInst
}

pub type VmmResult<T = ()> = Result<T, VmmError>;