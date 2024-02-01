use std::fmt::Display;
use std::io;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use crate::result::error::InitError;
use crate::result::IronResult;
use crate::runtime::GLOBAL_HAD_ALREADY_INITIALIZED;
use crate::sync::SyncStdoutLock;

#[derive(Clone)]
pub(crate) struct IronLogger
{
    stdout: Arc<Mutex<BufWriter<SyncStdoutLock>>>
}

impl IronLogger
{
    pub(crate) fn new() -> IronResult<Self>
    {
        //确认是否已经初始化过了
        unsafe {
            if GLOBAL_HAD_ALREADY_INITIALIZED
            {
                return Err(Box::new(InitError::HadAlreadyInitialize))
            }

            GLOBAL_HAD_ALREADY_INITIALIZED = true
        }

        //初始化stdout
        let stdout = Arc::new(Mutex::new(BufWriter::new(SyncStdoutLock::new())));

        Ok(
            Self
            {
                stdout
            }
        )
    }

    #[allow(dead_code)]
    #[inline]
    pub fn log<D:Display>(&mut self,log:D)
    {
        writeln!(self.stdout.lock().unwrap(),"{}",log).unwrap();
    }
    
    #[inline]
    pub fn flush(&mut self) -> io::Result<()>
    {
        self.stdout.lock().unwrap().flush()
    }
}

impl Drop for IronLogger {
    fn drop(&mut self)
    {
        self.flush().unwrap();
        unsafe {
            GLOBAL_HAD_ALREADY_INITIALIZED = false
        }
    }
}