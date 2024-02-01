use std::fmt::{Debug, Formatter};
use crate::result::marker::IronLoggerError;

#[derive(Clone, Copy)]
pub enum InitError
{
    HadAlreadyInitialize,
}

impl Debug for InitError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            &InitError::HadAlreadyInitialize => f.pad("HadAlreadyInitialize")
        }
    }
}

impl IronLoggerError for InitError {}