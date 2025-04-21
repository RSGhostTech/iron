use std::{fs::File, io::{self, BufWriter}, path::Path, sync::{mpsc::Sender, mpsc}, thread::{spawn, JoinHandle}};

use log::Level;

#[derive(Clone, Debug)]
pub(crate) enum LoggingCommand {
    Log(Level,String),
    Exit
}

pub struct Logger {
    sender:Sender<LoggingCommand>,
    handle:JoinHandle<()>
}

impl Logger {
    pub fn new<P:AsRef<Path>>(log_file:P) -> io::Result<Self> {
        let (sender,receiver) = mpsc::channel();
        let io = BufWriter::new(File::create_new(log_file)?);
        let handle = spawn(move || {
            let (mut receiver,mut io) = (receiver,io);
            loop {
                if let Ok(recv) = receiver.recv() {
                    match recv {
                        LoggingCommand::Log(level, text) => {
                            todo!()
                        },
                        LoggingCommand::Exit => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });
        Ok(Self {
            sender,
            handle
        })
    }
}