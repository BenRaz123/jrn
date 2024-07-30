use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    path::Path,
    process::exit,
    str::FromStr,
};

use crate::{
    date::Date,
    db::{LoadError, State},
    encryptor::Encryptor,
    fail,
};

use enum_display::EnumDisplay;
use enum_utils::FromStr;
use requestty::{prompt_one, Question};
use std::cmp::Ord;

const MASK_CHAR: char = '*';

pub enum AppResult {
    ChangedState,
    DidntChangeState,
    Quit,
}

#[derive(EnumDisplay, Debug, FromStr, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[enum_display(case = "Title")]
#[enumeration(rename_all = "PascalCase")]
enum PathWay {
    #[enumeration(rename = "Change Password")]
    ChangePassword,
    List,
    View,
    Edit,
    #[enumeration(rename = "View Today")]
    ViewToday,
    #[enumeration(rename = "Edit Today")]
    EditToday,
    Quit,
}
/// hello
pub fn init<E: Encryptor>(jrn_path: &str, e: &E) -> State {
    let mut state = State::new();
    if !Path::new(jrn_path).exists() {
        let pass = get_new_password();
        state.change_password(&pass);
        return state;
    }

    let mut pass = password("Please enter password");
    let mut loaded = state.load(jrn_path, &pass, e);

    if let Err(LoadError::IncorrectPassword) = loaded {
        loop {
            pass = password("Try Again. Please enter password");
            loaded = state.load(jrn_path, &pass, e);

            if let Err(LoadError::IncorrectPassword) = loaded {
            } else {
                break;
            }
        }
    }

    if let Err(e) = loaded {
        fail!("load error: {e:?}");
    }

    state
}

pub fn app_loop(state: &mut State) -> AppResult {
    let mut ret = AppResult::DidntChangeState;
    loop {
        let ar = app(state);
        match ar {
            AppResult::ChangedState => { ret = AppResult::ChangedState; },
            AppResult::Quit => { break; },
            _ => { continue; }
        }
    }
    ret
}

pub fn app(state: &mut State) -> AppResult {
    let pathways: HashSet<PathWay> = HashSet::from([
        PathWay::ChangePassword,
        PathWay::List,
        PathWay::View,
        PathWay::Edit,
        PathWay::ViewToday,
        PathWay::EditToday,
        PathWay::Quit,
    ]);
    let pathway = choose(pathways, "Welcome to jrn. Please choose a course of action");

    match pathway {
        PathWay::ChangePassword => change_password(state),
        PathWay::List => list_entries(&state),
        PathWay::View => view_entries(&state),
        PathWay::Edit => edit_entry(state),
        PathWay::ViewToday => view_today(&state),
        PathWay::EditToday => edit_today(state),
        PathWay::Quit => AppResult::Quit,
    }
}

fn edit_today(state: &mut State) -> AppResult {
    let content = state.get_today();
    let new_content = edit(content.as_deref(), ".org", "Press <Enter> to edit");

    state.set_today(&new_content);

    if content.is_some() {
        if content.unwrap() == new_content {
            return AppResult::DidntChangeState;
        }
    }

    AppResult::ChangedState
}

fn view_today(state: &State) -> AppResult {
    let entry = state.get_today().unwrap_or("<No Entry>".into());
    println!("{entry}");

    AppResult::DidntChangeState
}

fn edit_entry(state: &mut State) -> AppResult {
    let dates = HashSet::from_iter(state.entries.keys().map(|a| a.clone()));
    let date = choose(dates, "Which entry do you want to edit?");
    let old_content = state.get_entry(&date);
    if old_content.is_none() {
        fail!("couldn't get default content");
    }
    let new_content = edit(
        Some(&old_content.clone().unwrap()),
        ".org",
        "Press <Enter> to edit",
    );

    state.set_entry(&date, &new_content);

    match old_content.unwrap() == new_content {
        true => AppResult::DidntChangeState,
        false => AppResult::ChangedState,
    }
}

fn change_password(state: &mut State) -> AppResult {
    let old_password = state.password.clone();
    let new_password = get_new_password();
    state.change_password(&new_password);

    match old_password == new_password {
        true => AppResult::DidntChangeState,
        false => AppResult::ChangedState,
    }
}

fn view_entries(state: &State) -> AppResult {
    let dates: HashSet<Date> = HashSet::from_iter(state.entries.keys().map(|a| a.clone()));
    let date = choose(dates, "Please choose an entry");
    let entry = state.get_entry(&date);

    if entry.is_none() {
        fail!("invalid date entered");
    }

    println!("{}", entry.unwrap());

    AppResult::DidntChangeState
}

fn list_entries(state: &State) -> AppResult {
    let mut keys = state.entries.keys().collect::<Vec<_>>();
    keys.sort();
    for key in keys {
        println!("- {key}");
    }

    AppResult::DidntChangeState
}

fn confirmation(message: &str) -> bool {
    let question = Question::confirm(message)
        .message(format!("{message} (y/n)"))
        .build();

    let answer = prompt_one(question);

    if let Err(e) = answer {
        fail!("couldn't prompt: {e:?}");
    }

    let result = answer.unwrap().as_bool();

    if let None = result {
        fail!("coudln't get value from question");
    }

    result.unwrap()
}

fn password(message: &str) -> String {
    let question = Question::password(message)
        .message(message)
        .mask(MASK_CHAR)
        .build();

    let answer = prompt_one(question);

    if let Err(e) = answer {
        fail!("couldn't prompt: {e:?}");
    }

    let password = answer.unwrap();
    let password = password.as_string();

    if password.is_none() {
        fail!("failed to retreive prompt data");
    }

    password.unwrap().to_string()
}

fn edit(content: Option<&str>, filetype: &str, message: &str) -> String {
    let question = Question::editor(message)
        .message(message)
        .extension(filetype)
        .default(content.unwrap_or_default())
        .build();
    let answer = prompt_one(question);
    if let Err(e) = answer {
        fail!("couldn't prompt: {e:?}");
    }
    let string = answer.unwrap();
    let string = string.as_string();
    if string.is_none() {
        fail!("could not convert to string");
    }

    string.unwrap().into()
}

fn choose<T: Display + FromStr + Ord>(content: HashSet<T>, message: &str) -> T
where
    <T as FromStr>::Err: Debug,
{
    let mut content_as_vec = content.into_iter().collect::<Vec<T>>();
    content_as_vec.sort();
    let content = content_as_vec.into_iter().map(|x| x.to_string());
    let question = Question::select(message)
        .message(message)
        .choices(content)
        .build();

    let answer = prompt_one(question);

    if let Err(e) = answer {
        fail!("couldn't prompt: {e:?}");
    }

    let list_item = answer.unwrap();
    let list_item = list_item.as_list_item();

    if list_item.is_none() {
        fail!("couldn't get list item");
    }

    let value = T::from_str(&list_item.unwrap().text);

    if let Err(e) = value {
        fail!("couldn't get value from str: {e:?}");
    }

    value.unwrap()
}

fn get_new_password() -> String {
    let mut pass1 = password("New password please");
    let mut pass2 = password("Please repeat password");

    if pass1 != pass2 {
        loop {
            pass1 = password("Try again. New password please");
            pass2 = password("Please repeat password");

            if pass1 == pass2 {
                break;
            }
        }
    }

    pass1
}
