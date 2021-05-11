use std::io::{self, Write};
use std::sync::Mutex;

extern crate chrono;
#[macro_use]
extern crate slog;
extern crate slog_filerotate;
extern crate slog_term;
extern crate slog_scope;

use slog::{Drain, Record};
use slog_filerotate::FileAppender;
use slog_term::{CountingWriter, RecordDecorator, ThreadSafeTimestampFn};

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.9f";

pub const BITE: u64 = 1;
pub const KB: u64 = BITE * 1024;
pub const MB: u64 = KB * 1024;
pub const GB: u64 = MB * 1024;

#[macro_export]
macro_rules! crit( ($($args:tt)+) => {
    slog_scope::crit![$($args)+];
    std::process::exit(-1);
};);

#[macro_export]
macro_rules! error( ($($args:tt)+) => {
    slog_scope::error![$($args)+]
};);

#[macro_export]
macro_rules! warn( ($($args:tt)+) => {
    slog_scope::warn![$($args)+]
};);

#[macro_export]
macro_rules! info( ($($args:tt)+) => {
    slog_scope::info![$($args)+]
};);

#[macro_export]
macro_rules! debug( ($($args:tt)+) => {
    slog_scope::debug![$($args)+]
};);

#[macro_export]
macro_rules! trace( ($($args:tt)+) => {
    slog_scope::trace![$($args)+]
};);

fn timestamp_custom(io: &mut dyn io::Write) -> io::Result<()> {
    write!(io, "{}", chrono::Local::now().format(TIMESTAMP_FORMAT))
}

fn custom_print_msg_header(
    fn_timestamp: &dyn ThreadSafeTimestampFn<Output = io::Result<()>>,
    mut rd: &mut dyn RecordDecorator,
    record: &Record,
    use_file_location: bool,
) -> io::Result<bool> {
    rd.start_timestamp()?;
    fn_timestamp(&mut rd)?;

    rd.start_whitespace()?;
    write!(rd, " ")?;

    rd.start_level()?;
    write!(rd, "[{:<8}]", record.level().as_str())?;
    if use_file_location {
        rd.start_whitespace()?;
        write!(rd, " ")?;
        rd.start_location()?;
        write!(
            rd,
            "[{}:{}]",
            record.location().file,
            record.location().line,
        )?;
    }
    rd.start_whitespace()?;
    write!(rd, " ")?;

    rd.start_msg()?;
    let mut count_rd = CountingWriter::new(&mut rd);
    write!(count_rd, "{}", record.msg())?;
    Ok(count_rd.count() != 0)
}

pub fn initlogger(
    duplicate: bool,
    logfile: &str,
    filesize: u64,
    debug: bool,
    detail: bool,
) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let mut iner = slog_term::FullFormat::new(decorator)
        .use_custom_timestamp(timestamp_custom)
        .use_custom_header_print(custom_print_msg_header);
    if detail {
        iner = iner.use_file_location();
    }
    let drain = Mutex::new(iner.build());
    let drain_filter;
    if !debug {
        drain_filter = slog::LevelFilter::new(drain, slog::Level::Info);
    } else {
        drain_filter = slog::LevelFilter::new(drain, slog::Level::Trace);
    }
    if duplicate {
        let adapter = FileAppender::new(logfile, false, filesize, 2, true);
        let decorator_file = slog_term::PlainSyncDecorator::new(adapter);
        let mut file_iner = slog_term::FullFormat::new(decorator_file)
            .use_custom_timestamp(timestamp_custom)
            .use_custom_header_print(custom_print_msg_header);
        if detail {
            file_iner = file_iner.use_file_location();
        }

        let drain_file = file_iner.build();
        let drain_file_filter;
        if !debug {
            drain_file_filter = slog::LevelFilter::new(drain_file, slog::Level::Info);
        } else {
            drain_file_filter = slog::LevelFilter::new(drain_file, slog::Level::Trace);
        }

        slog::Logger::root(
            slog::Duplicate::new(drain_file_filter, drain_filter).fuse(),
            o!(),
        )
    } else {
        slog::Logger::root(drain_filter.fuse(), o!())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
