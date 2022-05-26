use std::io::{self, Write};
use std::sync::Mutex;

extern crate chrono;
#[macro_use]
extern crate slog;
extern crate slog_filerotate;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;

use slog::{Drain, Record};
use slog_filerotate::FileAppender;
pub use slog_scope::{
    crit as slog_scope_crit, debug as slog_scope_debug, error as slog_scope_error,
    info as slog_scope_info, trace as slog_scope_trace, warn as slog_scope_warn,
};
use slog_term::{CountingWriter, RecordDecorator, ThreadSafeTimestampFn};

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.9f";

pub const BITE: u64 = 1;
pub const KB: u64 = BITE * 1024;
pub const MB: u64 = KB * 1024;
pub const GB: u64 = MB * 1024;

#[macro_export]
macro_rules! crit( ($($args:tt)+) => {
    $crate::slog_scope_crit![$($args)+];
    std::process::exit(-1);
};);

#[macro_export]
macro_rules! error( ($($args:tt)+) => {
    $crate::slog_scope_error![$($args)+]
};);

#[macro_export]
macro_rules! warn( ($($args:tt)+) => {
    $crate::slog_scope_warn![$($args)+]
};);

#[macro_export]
macro_rules! info( ($($args:tt)+) => {
    $crate::slog_scope_info![$($args)+]
};);

#[macro_export]
macro_rules! debug( ($($args:tt)+) => {
    $crate::slog_scope_debug![$($args)+]
};);

#[macro_export]
macro_rules! trace( ($($args:tt)+) => {
    $crate::slog_scope_trace![$($args)+]
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

fn initlogger(
    duplicate: bool,
    logfile: &str,
    filesize: u64,
    debug: bool,
    detail: bool,
    verbose: bool,
    keep_num: usize,
    compress: bool,
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
        if verbose {
            drain_filter = slog::LevelFilter::new(drain, slog::Level::Trace);
        } else {
            drain_filter = slog::LevelFilter::new(drain, slog::Level::Debug);
        }
    }
    if duplicate {
        let adapter = FileAppender::new(logfile, false, filesize, keep_num, compress);
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
            if verbose {
                drain_file_filter = slog::LevelFilter::new(drain_file, slog::Level::Trace);
            } else {
                drain_file_filter = slog::LevelFilter::new(drain_file, slog::Level::Debug);
            }
        }

        slog::Logger::root(
            slog::Duplicate::new(drain_file_filter, drain_filter).fuse(),
            o!(),
        )
    } else {
        slog::Logger::root(drain_filter.fuse(), o!())
    }
}

pub fn setup_logger(
    duplicate: bool,
    logfile: &str,
    filesize: u64,
    debug: bool,
    detail: bool,
    verbose: bool,
    keep_num: usize,
    compress: bool,
) {
    let logger = initlogger(
        duplicate, logfile, filesize, debug, detail, verbose, keep_num, compress,
    );
    let guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    guard.cancel_reset();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
