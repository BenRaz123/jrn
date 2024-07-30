//! a journal app
#![warn(missing_docs)]

use db::State;
use encryptor::{Secure, ZeroSecurity};
use ui::{app_loop, AppResult};

pub mod date;
pub mod db;
pub mod encryptor;
pub mod fail;
pub mod ui;

fn main() {
    let mut state = ui::init("heyhey.json", &Secure);
    let app_result = app_loop(&mut state);
    if let AppResult::ChangedState = app_result {
        let _ = state.save("heyhey.json", &Secure);
    }
}
