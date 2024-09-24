//! crate for the `fail!` macro


#[macro_export]
/// print an error message and exit with code 1
macro_rules! fail {
    ($msg:expr) => {
        let err = format!($msg);
        eprintln!("{}", color_print::cformat!("<red,bold>error:</> {err}"));
        std::process::exit(1);
    };
}
