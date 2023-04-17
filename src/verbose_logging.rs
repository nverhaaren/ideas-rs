#[allow(dead_code)]
pub enum VerboseLoggingBackend {
    Noop,
    Println,
    EPrintln,
    // FUTURE: standard logging library backends, custom backends
}

#[macro_export]
macro_rules! verbose_log {
    ($($e:expr),*) => {match verbose_logging_backend() {
        $crate::verbose_logging::VerboseLoggingBackend::Noop => {},
        $crate::verbose_logging::VerboseLoggingBackend::Println => {println!($($e),*)},
        $crate::verbose_logging::VerboseLoggingBackend::Eprintln => {eprintln!($($e),*)},
    }};
}

#[macro_export]
macro_rules! verbose_logging {
    () => {};
    (true) => {verbose_logging!(Println);};
    (false) => {verbose_logging!(Noop);};
    ($i:ident) => {
        #[allow(dead_code)]
        const fn verbose_logging_backend() -> $crate::verbose_logging::VerboseLoggingBackend {
            $crate::verbose_logging::VerboseLoggingBackend::$i
        }
    };
}