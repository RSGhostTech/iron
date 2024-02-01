use crate::prelude::IronLoggerError;

pub mod error;
pub mod marker;

pub type IronResult<T> = Result<T,Box<dyn IronLoggerError>>;