//! module for command line arguments
use argh::FromArgs;

use crate::date::Date;

#[derive(FromArgs, PartialEq, Debug, Clone)]
/// a journal app
pub struct Arguments {
    /// dump default config to stdout and exit
    #[argh(switch, short='d')]
    pub dump_default_config: bool,

    /// use toml file for config (otherwise looks for config file at `$JRN_CONFIG_PATH`,
    /// `$XDG_CONFIG_DIR/jrn/config.toml`, or `~/.config/jrn/config.toml`)
    #[argh(option, short = 'c')]
    pub config_file: Option<String>,

    /// use following as password (insecure, not recomended)
    #[argh(option, short = 'p')]
    pub password: Option<String>,

    /// read password from file (recomended)
    #[argh(option, short = 'P')]
    pub password_file: Option<String>,
    
    /// force ui not to loop
    #[argh(switch, short = 'D')]
    pub dont_loop: bool,

    /// force ui to loop
    #[argh(switch, short = 'L')]
    pub do_loop: bool,

    /// file type for journal entries (default is ".md")
    #[argh(option, short = 'F')]
    pub file_type: Option<String>,

    /// file path to be used for storing journal (default is either 
    /// "$JRN_JOURNAL" or "./jrn.json")
    #[argh(option, short = 'f')]
    pub file_path: Option<String>,

    #[argh(subcommand)]
    /// action taken
    pub subcommand: Option<SubCommand>,
}

#[derive(FromArgs, PartialEq, Debug, Clone)]
#[argh(subcommand)]
/// An optional subcommand with optional information
pub enum SubCommand {
    /// Intent to change password as well as the new password (optional)
    ChangePassword(ChangePassword),
    /// Intent to list entries (no options)
    List(List),
    /// Intent to view entries as well as the index of the entry (optional)
    View(View),
    /// Intent to edit entries as well as the index of the entry to edit and the new content (both optional)
    Edit(Edit),
    /// The intent to view today's entry (no options)
    ViewToday(ViewToday),
    /// The intent to edit today's entry as well as the new content (optional) 
    EditToday(EditToday)
}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// change password
#[argh(subcommand, name = "change-password")]
pub struct ChangePassword {
    #[argh(option, short = 'n')]
    /// the new password in string form (vulnerable to shell history attacks, not recomended)
    pub new_password: Option<String>,

    #[argh(option, short = 'N')]
    /// the new password stored in a file (more secure but less secure that interactive auth)
    pub new_password_file: Option<String>
}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// List by date
#[argh(subcommand, name = "list")]
pub struct List {}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// Get entry
#[argh(subcommand, name = "view")]
pub struct View {
    #[argh(positional)]
    /// the date of the entry (In YYYY-MM-DD format or today-n format)
    pub date: Option<Date>
}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// Edit entry
#[argh(subcommand, name = "edit")]
pub struct Edit {
    /// the date of the entry (In YYYY-MM-DD format or today-n format)
    #[argh(positional)]
    pub date: Option<Date>,

    /// the content to write, in string form
    #[argh(option, short =  'c')]
    pub content: Option<String>,

    /// file path to write
    #[argh(option, short = 'C')]
    pub content_path: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// view today's entry
#[argh(subcommand, name="view-today")]
pub struct ViewToday {}

#[derive(FromArgs, PartialEq, Debug, Clone, Default)]
/// edit today's entry
#[argh(subcommand, name="edit-today")]
pub struct EditToday {
    /// content to write, in string form
    #[argh(option, short = 'c')]
    pub content: Option<String>,

    /// file path to write
    #[argh(option, short = 'C')]
    pub content_path: Option<String>,
}
