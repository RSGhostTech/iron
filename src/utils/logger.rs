use std::collections::VecDeque;
use std::io::{BufWriter, prelude::*};

#[derive(Debug, Clone)]
pub struct Logger {
    buf: VecDeque<String>,
    file: 
}

impl Logger {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new()
        }
    }
}