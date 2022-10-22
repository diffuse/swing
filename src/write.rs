use colored::Colorize;
use log::Level;
use std::io;
use std::io::Write;
use std::sync::Mutex;

/// Write logs to stdout/stderr with synchronization
pub struct LogWriter {
    /// switch for enabling log splitting to `stderr`
    ///
    /// - `true`: log `trace` - `info` levels to `stdout` and `warn` - `error` levels to `stderr`
    ///
    /// - `false`: log all levels to `stdout`
    use_stderr: bool,
    /// guard against interleaving from simultaneous writes to stdout + stderr
    write_mtx: Mutex<()>,
    /// handle to stdout
    stdout: io::Stdout,
    /// handle to stderr
    stderr: io::Stderr,
}

impl LogWriter {
    /// Create a new LogWriter
    pub fn new(use_stderr: bool) -> LogWriter {
        LogWriter {
            use_stderr,
            write_mtx: Mutex::new(()),
            stdout: io::stdout(),
            stderr: io::stderr(),
        }
    }

    /// Write log message to output destination
    ///
    /// # Arguments
    ///
    /// * `msg` - the log message to write
    /// * `level` - the level of this log message
    pub fn write(&self, msg: String, level: Level) {
        // stdout and stderr already have their own locks, but
        // there is nothing preventing logs simultaneously written
        // to stdout + stderr from being interleaved in the console
        //
        // this guard synchronizes writes so that stdout will not be
        // interleaved with stderr
        let _lk = self.write_mtx.lock().unwrap();

        match level {
            Level::Warn | Level::Error => {
                if self.use_stderr {
                    let _ = writeln!(self.stderr.lock(), "{}", msg.bold());
                } else {
                    let _ = writeln!(self.stdout.lock(), "{}", msg.bold());
                }
            }
            _ => {
                let _ = writeln!(self.stdout.lock(), "{}", msg);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_handles_empty_msg() {
        let levels = vec![
            Level::Trace,
            Level::Debug,
            Level::Info,
            Level::Warn,
            Level::Error,
        ];

        // using stderr
        let writer = LogWriter::new(true);

        for level in levels.iter() {
            writer.write("".to_string(), *level);
        }

        // not using stderr
        let writer = LogWriter::new(false);

        for level in levels.iter() {
            writer.write("".to_string(), *level);
        }
    }
}
