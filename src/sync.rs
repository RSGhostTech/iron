use std::borrow::{Borrow, BorrowMut};
use std::io::{stdout, StdoutLock, Write};

pub(crate) struct SyncStdoutLock
{
    wrap:StdoutLock<'static>
}

unsafe impl Sync for SyncStdoutLock {}
unsafe impl Send for SyncStdoutLock {}

impl Borrow<StdoutLock<'static>> for SyncStdoutLock
{
    fn borrow(&self) -> &StdoutLock<'static>
    {
        &self.wrap
    }
}

impl BorrowMut<StdoutLock<'static>> for SyncStdoutLock
{
    fn borrow_mut(&mut self) -> &mut StdoutLock<'static>
    {
        &mut self.wrap
    }
}

impl SyncStdoutLock
{
    pub fn new() -> SyncStdoutLock
    {
        SyncStdoutLock
        {
            wrap : stdout().lock()
        }
    }
}

impl Write for SyncStdoutLock
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> 
    {
        self.wrap.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> 
    {
        self.wrap.flush()
    }
}