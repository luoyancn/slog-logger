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

pub const BITE: u64 = 1;
pub const KB: u64 = BITE * 1024;
pub const MB: u64 = KB * 1024;
pub const GB: u64 = MB * 1024;

fn timestamp_custom(io: &mut dyn io::Write) -> io::Result<()> {
    write!(io, "{}", chrono::Local::now().format(TIMESTAMP_FORMAT))
}

pub fn initlogger(
    duplicate: bool,
    logfile: &str,
    filesize: u64,
    debug: bool,
    detail: bool,
) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(
        slog_term::FullFormat::new(decorator)
            .use_custom_timestamp(timestamp_custom)
            .build(),
    );
    let mut drain_filter = slog::LevelFilter::new(drain, slog::Level::Trace);
    if !debug {
        drain_filter = drain_filter.filter_level(slog::Level::Info).0;
    }
    if duplicate {
        let adapter = FileAppender::new(logfile, false, filesize, 2, true);
        let decorator_file = slog_term::PlainSyncDecorator::new(adapter);
        let drain_file = slog_term::FullFormat::new(decorator_file)
            .use_custom_timestamp(timestamp_custom)
            .build();
        let mut drain_file_filter = slog::LevelFilter::new(drain_file, slog::Level::Trace);
        if !debug {
            drain_file_filter = drain_file_filter.filter_level(slog::Level::Info).0;
        }

        if !detail {
            slog::Logger::root(
                slog::Duplicate::new(drain_file_filter, drain_filter).fuse(),
                o!(),
            )
        } else {
            slog::Logger::root(
                slog::Duplicate::new(drain_file_filter, drain_filter).fuse(),
                o!("place" => FnValue( move |info| { format!("{}:{} {}", info.file(), info.line(), info.module())})),
            )
        }
    } else {
        if !detail {
            slog::Logger::root(drain_filter.fuse(), o!())
        } else {
            slog::Logger::root(
                drain_filter.fuse(),
                o!("place" => FnValue( move |info| { format!("{}:{} {}", info.file(), info.line(), info.module())})),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
