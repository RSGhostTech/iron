pub use log::*;
use std::cell::OnceCell;
use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread::spawn;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub enum IOTextWrapper {
    Log(String),
    Shutdown,
}
pub(super) mod functions {
    use super::Record;
    use crate::utils::logger::IOTextWrapper;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    use std::sync::mpsc::Receiver;

    /// 用于格式化log文本信息
    #[inline]
    pub(super) fn format_log_text(record: &Record) -> String {
        format!(
            "[{} {} {}] {}",
            chrono::Local::now().to_utc().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.module_path().unwrap_or("UnknownModule"),
            record.args()
        )
    }

    /// 用于io工作线程写入log
    #[inline]
    pub(super) fn io_log(receiver: Receiver<IOTextWrapper>, mut io: BufWriter<File>) {
        for cmd in receiver.into_iter() {
            match cmd {
                IOTextWrapper::Shutdown => return,
                IOTextWrapper::Log(mut log) => {
                    log.push_str("\r\n");
                    io.write_all(log.as_bytes()).unwrap();
                    io.flush().unwrap();
                }
            }
        }
    }

    /// 默认log文件名
    #[inline]
    pub(super) fn filename() -> String {
        format!("{}.log", chrono::Local::now().format("%Y%m%d_%H%M%S"))
    }
}
pub struct Logger {
    handle: Mutex<OnceCell<JoinHandle<()>>>,
    sender: Sender<IOTextWrapper>,
}
impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        let text = functions::format_log_text(record);
        println!("{}", text);
        self.sender.send(IOTextWrapper::Log(text)).unwrap()
    }
    fn flush(&self) {
        self.sender.send(IOTextWrapper::Shutdown).unwrap();
        self.handle.lock().unwrap().take().unwrap().join().unwrap();
    }
}

impl Logger {
    pub fn init_logger() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let io = BufWriter::new(File::create_new(functions::filename()).unwrap());
        let handle = Mutex::new(OnceCell::from(spawn(move || {
            functions::io_log(receiver, io)
        })));

        set_boxed_logger(Box::new(Self { handle, sender })).unwrap();
        set_max_level(LevelFilter::Info);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log() {
        Logger::init_logger();
        warn!("This is a warning");
        info!("This is a info message");
    }
}
