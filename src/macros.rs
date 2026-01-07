#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum LogLevel {
    ERROR,
    WARN,
    LOG,
    DEBUG,
}

macro_rules! notify {
    ($msg:expr, $severity:expr, $log_level:ident$(, $var:ident)?) => {
        if $severity <= *$log_level {
            let line = line!();
            let file = file!();

            match $severity {
                LogLevel::LOG => {
                    print!("[LOG]: {}.", $msg);
                }
                LogLevel::WARN => {
                    print!("[WARN]: {}.", $msg);
                }
                LogLevel::DEBUG => {
                    print!("in [{}] at [{}] [DEBUG]: {}.", file, line, $msg);
                }
                LogLevel::ERROR => {
                    print!("in [{}] at [{}] [ERROR]: {}.", file, line, $msg);
                }
            }

            $( print!(" {}\n", $var); )?
        }
    };
}
