//! a journal app
#![warn(missing_docs)]

use std::process::exit;

use cli::Arguments;
use config::Config;
use db::State;
use encryptor::{Secure, ZeroSecurity};
use ui::{app, AppResult};

pub mod date;
pub mod db;
pub mod encryptor;
pub mod fail;
pub mod ui;
pub mod cli;
pub mod config;

fn main() {
    let args: Arguments = argh::from_env();
    let config: Config = Config::get_config(&args);
    if args.dump_default_config {
        let to_print = toml::to_string_pretty(&Config::default());
        if to_print.is_err() {
            fail!("couldn't serialize config");
        }
        println!("{}", to_print.unwrap());
        exit(0);
    }
    let file = config.clone().file_path.unwrap_or("./jrn.json".into());

    if let (Some(dont_loop), Some(do_loop)) = (config.dont_loop, config.do_loop) {
        if dont_loop && do_loop {
            fail!("can't both loop and not loop");
        }
    }

    if config.password.is_some() && config.password_file.is_some() {
        fail!("please give only one password");
    }

    let mut state = ui::init(&config, &Secure);

    let app_result = app(&config, args.subcommand, &mut state);
    if let AppResult::ChangedState = app_result {
        let save = state.save(&file, &Secure);
        if let Err(e) = save {
            fail!("error saving: {e:?}");
        }
    }
}
