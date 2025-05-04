pub use log::*;
use std::fs::File;
use std::io::BufWriter;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::spawn;
use std::thread::JoinHandle;

#[macro_export]
macro_rules! init_logger {
    () => {
        let _inited_logger_guard = Arc::new(Logger::init_logger());
    };
}

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
    use std::time::Instant;

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
        let mut timer = Instant::now();

        let write_buffer = |io: &mut BufWriter<File>, buffer: &mut Vec<u8>| {
            io.write_all(buffer).unwrap();
            io.flush().unwrap();
            buffer.clear();
        };

        for cmd in receiver {
            match cmd {
                IOTextWrapper::Shutdown => {
                    write_buffer(&mut io, &mut buffer);
                    break;
                }
                IOTextWrapper::Log(log) => {
                    if write!(&mut buffer, "{log}{NEXT}").is_err()
                        || buffer.len() >= 8000
                        || timer.elapsed().as_secs() >= 10
                    {
                        write_buffer(&mut io, &mut buffer);
                        timer = Instant::now();
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

#[derive(Debug)]
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
    }
}

impl Logger {
    /// 初始化Logger，并创建一个守卫，使得能在某个的作用域中一直存在。
    ///
    /// 为了使得Logger一直能够生效，请为guard绑定一个变量名，或者使用init_logger!()来隐式绑定变量名
    pub fn init_logger() -> LoggerGuard {
        let (sender, receiver) = std::sync::mpsc::channel();
        let io = BufWriter::new(File::create_new(functions::filename()).unwrap());
        let handle = spawn(move || functions::io_log(receiver, io));

        let arc = Arc::new(Self { handle, sender });

        set_boxed_logger(Box::new(arc.clone())).unwrap();
        set_max_level(LevelFilter::Info);
        LoggerGuard(Some(arc))
    }

    /// 刷新并释放io流
    pub fn free(&self) {
        self.flush();
        while self.handle.is_finished() {}
    }
}

pub struct LoggerGuard(Option<Arc<Logger>>);
impl Drop for LoggerGuard {
    fn drop(&mut self) {
        self.0.take().unwrap().free();
    }
}
