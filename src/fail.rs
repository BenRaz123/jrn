#[macro_export]
macro_rules! fail {
    ($msg:expr) => {
        let err = format!($msg);
        eprintln!("{}", color_print::cformat!("<red,bold>error:</> {err}"));
        std::process::exit(1);
    };
}
