use std::{
    collections::HashSet, fmt::{write, Debug, Display}, path::Path, process::exit, str::FromStr
};

use crate::{
    cli::{
        Arguments, ChangePassword, Edit, EditToday, SubCommand, View,
    }, config::Config, date::Date, db::{LoadError, State}, encryptor::Encryptor, fail
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
pub enum PathWay {
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

#[derive(Debug, EnumDisplay)]
pub enum SubCommandFromPathWayError {
    QuitVariantWasUsed,
}

impl TryFrom<PathWay> for SubCommand {
    type Error = SubCommandFromPathWayError;
    fn try_from(value: PathWay) -> Result<Self, Self::Error> {
        use PathWay as PW;
        use SubCommand as SC;
        match value {
            PW::ChangePassword => Ok(SC::ChangePassword(Default::default())),
            PW::List => Ok(SC::List(Default::default())),
            PW::View => Ok(SC::View(Default::default())),
            PW::Edit => Ok(SC::Edit(Default::default())),
            PW::ViewToday => Ok(SC::ViewToday(Default::default())),
            PW::EditToday => Ok(SC::EditToday(Default::default())),
            PW::Quit => Err(SubCommandFromPathWayError::QuitVariantWasUsed),
        }
    }
}
/// hello
pub fn init<E: Encryptor>(config: &Config, e: &E) -> State {
    let config = config.clone();
    let jrn_path = config.file_path.as_deref().unwrap_or("./jrn.json");
    let mut state = State::new();
    if !Path::new(jrn_path).exists() {
        let pass = match (config.password, config.password_file) {
            (None, None) => get_new_password(),
            (Some(password), None) => password,
            (None, Some(password_file)) => {
                let password = std::fs::read_to_string(&password_file);
                if let Err(e) = password {
                    fail!("couldn't read password from file {password_file}: {e:?}");
                }
                password.unwrap().trim().into()
            }
            (Some(_), Some(_)) => {
                fail!("can't give both password string and password file");
            }
        };
        state.change_password(&pass);
        return state;
    }

    let mut pass = match (config.password, config.password_file) {
        (None, None) => password("Please enter your password"),
        (Some(password), None) => password,
        (None, Some(password_file)) => {
            let password = std::fs::read_to_string(&password_file);
            if let Err(e) = password {
                fail!(
                    "couldn't read password from file {password_file}: {e:?}"
                );
            }
            password.unwrap().trim().into()
        }
        (Some(_), Some(_)) => {
            fail!("can't give both password string and password file");
        }
    };

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

pub fn should_loop(config: &Config, subcommand: &Option<SubCommand>) -> bool {
    let config = config.clone();
    if let Some(do_loop) = config.do_loop {
        if do_loop {
            return true;
        }
    }
    if let Some(dont_loop) = config.dont_loop {
        if dont_loop {
            return false;
        }
    }
    if subcommand.is_some() {
        return false;
    }

    true
}

pub fn app(config: &Config, subcommand: Option<SubCommand>, state: &mut State) -> AppResult {
    let should_loop = should_loop(config, &subcommand);
    if should_loop {
        let config = config.clone();
        let mut subcommand = subcommand.clone();
        let mut ret = AppResult::DidntChangeState;
        loop {
            let ar = _app(&config, subcommand, state);
            subcommand = None;
            match ar {
                AppResult::ChangedState => {
                    ret = AppResult::ChangedState;
                }
                AppResult::Quit => {
                    break;
                }
                _ => {
                    continue;
                }
            }
        }
        ret
    } else {
        return _app(config, subcommand, state);
    }
}

pub fn prompt_pathway() -> PathWay {
    let pathways: HashSet<PathWay> = HashSet::from([
        PathWay::ChangePassword,
        PathWay::List,
        PathWay::View,
        PathWay::Edit,
        PathWay::ViewToday,
        PathWay::EditToday,
        PathWay::Quit,
    ]);

    let pathway =
        choose(pathways, "Welcome to jrn. Please choose a course of action", false);

    pathway
}

fn _app(config: &Config, subcommand: Option<SubCommand>, state: &mut State) -> AppResult {
    use SubCommand as SC;

    let subcommand = match &subcommand {
        None => {
            let pw = prompt_pathway();
            let subcommand = SubCommand::try_from(pw);
            if subcommand.is_err() {
                return AppResult::Quit;
            }
            subcommand.unwrap()
        }
        Some(subcommand) => subcommand.clone(),
    };

    match subcommand {
        SC::ChangePassword(opts) => change_password(&opts, state),
        SC::List(_) => list_entries(&state),
        SC::View(opts) => view_entries(&opts, &state),
        SC::Edit(opts) => edit_entry(config, &opts, state),
        SC::ViewToday(_) => view_today(&state),
        SC::EditToday(opts) => edit_today(config, &opts, state),
    }
}

pub fn edit_today(config: &Config, opts: &EditToday, state: &mut State) -> AppResult {
    let opts = opts.clone();
    let config = config.clone();

    if opts.content.is_some() && opts.content_path.is_some() {
        fail!("can't give both content string and content path");
    }

    let content = match (opts.content, opts.content_path) {
        (None, None) => {
            let content = state.get_today();
            let new_content =
                edit(content.as_deref(), &config.file_type.unwrap_or(".md".into()), "Press <Enter> to edit");
            new_content
        }
        (Some(content), None) => content,
        (None, Some(content_path)) => {
            let content = std::fs::read_to_string(&content_path);
            if let Err(e) = content {
                fail!("couldn't read content from file {content_path}: {e:?}");
            }
            content.unwrap()
        }
        (Some(_), Some(_)) => {
            fail!("can't give both content string and content path");
        }
    };

    let old_content = state.get_today();
    state.set_today(&content);

    if old_content.is_some() {
        if old_content.unwrap() == content {
            return AppResult::DidntChangeState;
        }
    }

    AppResult::ChangedState
}

pub fn view_today(state: &State) -> AppResult {
    let entry = state.get_today().unwrap_or("<No Entry>".into());
    println!("{entry}");

    AppResult::DidntChangeState
}

pub fn edit_entry(config: &Config, opts: &Edit, state: &mut State) -> AppResult {
    let opts = opts.clone();
    let config = config.clone();

    let date = match opts.date {
        Some(date) => date,
        None => {
            let dates = get_dates(&state);
            choose(dates, "Which entry do you want to edit?", true)
        }
    };

    let old_content = state.get_entry(&date);

    let new_content = match (opts.content, opts.content_path) {
        (Some(_), Some(_)) => {
            fail!("can't give both content and content path");
        }
        (Some(content), None) => content,
        (None, Some(content_path)) => {
            let content = std::fs::read_to_string(&content_path);
            if let Err(e) = content {
                fail!("couldn't read content from file {content_path}: {e:?}");
            }
            content.unwrap()
        }
        (None, None) => {
            edit(
                old_content.as_deref(),
                config.file_type.as_deref().unwrap_or(".md"), 
                "Press <Enter> to edit",
            )
        }
    };

    state.set_entry(&date, &new_content);

    if old_content.is_some() {
        if old_content.unwrap() == new_content {
            return AppResult::DidntChangeState;
        }
    }

    AppResult::ChangedState
}

pub fn change_password(opts: &ChangePassword, state: &mut State) -> AppResult {
    let opts = opts.clone();

    let new_password = match (opts.new_password, opts.new_password_file) {
        (Some(_), Some(_)) => {
            fail!("can't give both new password and new password file");
        }
        (Some(new_password), None) => new_password,
        (None, Some(new_password_file)) => {
            let new_password = std::fs::read_to_string(&new_password_file);
            if let Err(e) = new_password {
                fail!("couldn't read content from file {new_password_file}: {e:?}");
            }
            new_password.unwrap()
        }
        (None, None) => get_new_password(),
    };

    let old_password = state.password.clone();
    state.change_password(&new_password);

    match old_password == new_password {
        true => AppResult::DidntChangeState,
        false => AppResult::ChangedState,
    }
}

pub fn view_entries(opts: &View, state: &State) -> AppResult {
    let opts = opts.clone();

    let date = match opts.date {
        Some(date) => date,
        None => {
            let dates = HashSet::from_iter(state.entries.keys().map(|a| a.clone()));
            if dates.is_empty() {
                println!("No entries to view!");
                exit(0)
            }
            choose(dates, "Please choose an entry", true)
        }
    };

    let entry = state.get_entry(&date);

    if entry.is_none() {
        fail!("invalid date entered");
    }

    println!("{}", entry.unwrap());

    AppResult::DidntChangeState
}

pub fn list_entries(state: &State) -> AppResult {
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

fn choose<T: Display + FromStr + Ord>(content: HashSet<T>, message: &str, reverse: bool) -> T
where
    <T as FromStr>::Err: Debug,
{
    let mut content_as_vec = content.into_iter().collect::<Vec<T>>();
    content_as_vec.sort();
    if reverse {
        content_as_vec.reverse();
    }
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

fn get_dates(state: &State) -> HashSet<Date>{
    let state_keys = HashSet::from_iter(state.entries.keys().map(|a| a.clone()));
    if state_keys.is_empty() {
        /*TODO Remove me! */ println!("aaa");
        return HashSet::from([Date::today()]);
    }

    state_keys
}
