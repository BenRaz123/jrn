use std::{env, path::Path};

use crate::cli::Arguments;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub password: Option<String>,
    pub password_file: Option<String>,
    pub dont_loop: Option<bool>,
    pub do_loop: Option<bool>,
    pub file_type: Option<String>,
    pub file_path: Option<String>,
}

impl From<Arguments> for Config {
    fn from(value: Arguments) -> Self {
        let Arguments {
            password,
            password_file,
            dont_loop,
            do_loop,
            file_type,
            file_path,
            ..
        } = value;
        Self {
            password,
            password_file,
            dont_loop: Some(dont_loop),
            do_loop: Some(do_loop),
            file_type,
            file_path,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let password = None;
        let password_file = None;
        let dont_loop = Some(false);
        let do_loop = Some(false);
        let file_type = Some(".md".into());
        let file_path = Some("./jrn.json".into());
        Self {
            password,
            password_file,
            dont_loop,
            do_loop,
            file_type,
            file_path,
        }
    }
}

impl Config {
    fn get_journal_path(default_config: &Self) -> String {
       match env::var("JRN_JOURNAL") {
            Ok(file) => { println!("{file}"); return file; },
            Err(_) => default_config.clone().file_path.expect("always Some(String) if returned by fn Self::get_default_config")
        }
    }
    fn get_config_path(args: &Arguments) -> Option<String> {
        let args = args.clone();

        if let Some(config_file) = args.config_file {
            return Some(config_file);
        }

        if let Ok(config_file) = env::var("JRN_CONFIG_FILE") {
            println!("Got it!");
            if Path::new(&config_file).exists() {
                return Some(config_file);
            }
        }

        if let Ok(config_dir) = env::var("XDG_CONFIG_DIR") {
            if Path::new(&format!("{config_dir}/jrn/config.toml")).exists() {
                return Some(format!("{config_dir}/jrn/config.toml"));
            }
        }

        if let Ok(home_dir) = env::var("HOME") {
            let cpath = format!("{home_dir}/.config/jrn/config.toml");
            if Path::new(&cpath).exists() {
                return Some(cpath);
            }
        }

        None
    }

    fn get_default_config(path: Option<&str>) -> Self {
        if path.is_none() {
            return Self::default();
        }

        let path_contents = std::fs::read_to_string(path.unwrap());

        if path_contents.is_err() {
            return Self::default();
        }

        let tml = toml::from_str(&path_contents.unwrap());

        match tml {
            Err(_) => Self::default(),
            Ok(tml) => tml,
        }
    }

    pub fn get_config(args: &Arguments) -> Self {
        let args = args.clone();
        let path = Self::get_config_path(&args);
        let default_config: Config = Self::get_default_config(path.as_deref());

        let password = match args.password {
            Some(_) => args.password,
            None => default_config.clone().password,
        };

        let password_file = match args.password_file {
            Some(_) => args.password_file,
            None => default_config.clone().password_file,
        };

        let do_loop = match args.do_loop {
            true => Some(args.do_loop),
            false => default_config.do_loop,
        };

        let dont_loop = match args.dont_loop {
            true => Some(args.dont_loop),
            false => default_config.dont_loop,
        };

        let file_type = match args.file_type {
            Some(_) => args.file_type,
            None => default_config.clone().file_type,
        };

        let file_path = match args.file_path {
            Some(_) => args.file_path,
            None => Some(Self::get_journal_path(&default_config)),
        };

        Self {
            password,
            password_file,
            do_loop,
            dont_loop,
            file_type,
            file_path,
        }
    }
}
