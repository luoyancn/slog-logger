use std::io;
use std::sync::Mutex;

extern crate chrono;
#[macro_use]
extern crate slog;
extern crate slog_filerotate;
extern crate slog_term;

use slog::Drain;
use slog::FnValue;
use slog_filerotate::FileAppender;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.9f";

fn timestamp_custom(io: &mut dyn io::Write) -> io::Result<()> {
    write!(io, "{}", chrono::Local::now().format(TIMESTAMP_FORMAT))
}

pub fn initlogger(duplicate: bool, logfile: &str, filesize: u64) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(
        slog_term::FullFormat::new(decorator)
            .use_custom_timestamp(timestamp_custom)
            .build(),
    );
    if duplicate {
        let adapter = FileAppender::new(logfile, false, filesize, 2, true);

        let decorator_file = slog_term::PlainSyncDecorator::new(adapter);
        let drain_file = slog_term::FullFormat::new(decorator_file)
            .use_custom_timestamp(timestamp_custom)
            .build();
        slog::Logger::root(
            slog::Duplicate::new(
                slog::LevelFilter::new(drain_file, slog::Level::Trace),
                slog::LevelFilter::new(drain, slog::Level::Trace),
            )
            .fuse(),
            o!("place" => FnValue( move |info| { format!("{}:{} {}", info.file(), info.line(), info.module())})),
        )
    } else {
        slog::Logger::root(
            slog::LevelFilter::new(drain, slog::Level::Trace).fuse(),
            o!("place" => FnValue( move |info| { format!("{}:{} {}", info.file(), info.line(), info.module())})),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
