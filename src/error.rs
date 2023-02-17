#[derive(Debug, PartialEq)]
pub enum VmmError {
    NotSupported,
    NoFound,
    Unimplemented,
    TranslationError
}

pub type VmmResult<T = ()> = Result<T, VmmError>;