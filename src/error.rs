#[derive(Debug, PartialEq)]
pub enum VmmError {
    NotSupported,
    NoFound,
    Unimplemented,
    TranslationError,
    DeviceNotFound,
    PseudoInst,
    DecodeInstError
}

pub type VmmResult<T = ()> = Result<T, VmmError>;