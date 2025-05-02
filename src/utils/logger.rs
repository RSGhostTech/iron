pub use log::*;
use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::Sender;
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

    #[cfg(target_os = "windows")]
    const NEXT: &'static str = "\r\n";
    #[cfg(not(target_os = "windows"))]
    const NEXT: &'static str = "\n";

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
        let mut buffer = Vec::with_capacity(8192);

        for cmd in receiver.into_iter() {
            match cmd {
                IOTextWrapper::Shutdown => {
                    io.write(&buffer).unwrap();
                    io.flush().unwrap();
                },
                IOTextWrapper::Log(log) => {
                    if write!(&mut buffer, "{log}{NEXT}").is_err() || buffer.len() >= 8000 {
                        io.write(&buffer).unwrap();
                        io.flush().unwrap();
                        buffer.clear();
                    }
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
    handle: JoinHandle<()>,
    sender: Sender<IOTextWrapper>,
}
impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    #[inline]
    fn log(&self, record: &Record) {
        let text = functions::format_log_text(record);
        println!("{}", text);
        if record.level() != Level::Info {
            self.sender.send(IOTextWrapper::Log(text)).unwrap()
        }
    }

    #[inline]
    fn flush(&self) {
        self.sender.send(IOTextWrapper::Shutdown).unwrap();
        while !self.handle.is_finished() {}
    }
}

impl Logger {
    pub fn init_logger() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let io = BufWriter::new(File::create_new(functions::filename()).unwrap());
        let handle = spawn(move || functions::io_log(receiver, io));

        set_boxed_logger(Box::new(Self { handle, sender })).unwrap();
        set_max_level(LevelFilter::Warn);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush()
    }
}
