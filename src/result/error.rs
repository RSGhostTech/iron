use std::fmt::{Debug, Formatter};

#[derive(Clone, Copy)]
pub enum InitError
{
    HadAlreadyInitialize,
}

impl AsRef<str> for InitError
{
    fn as_ref(&self) -> &'static str
    {
        match self
        {
            InitError::HadAlreadyInitialize => "HadAlreadyInitialize"
        }
    }
}

impl Debug for InitError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        f.pad(self.as_ref())
    }
}

