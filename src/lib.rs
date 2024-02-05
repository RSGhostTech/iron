mod result;
mod logger;
mod runtime;
mod sync;

pub use logger::IronLogger;
pub use result::IronResult;

#[allow(private_interfaces)]
#[inline]
pub fn init() -> IronResult<IronLogger>
{
    IronLogger::new()
}

#[allow(private_interfaces)]
#[inline]
pub fn free(logger:IronLogger)
{
    drop(logger)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn init_log()
    {
        let log = init().unwrap();
        drop(log);
        let _ = init().unwrap();
    }

    #[test]
    fn write_test()
    {
        let mut log = init().unwrap();
        log.log("Hello!")
    }
}