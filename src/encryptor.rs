pub trait Encryptor {
    fn hash_password(password: &str) -> String;
    fn verify_password<'a>(hashed_password: &'a str, entered_password: &'a str) -> bool;
    fn encrypt_journal_entry<'a>(password: &'a str, entry: &'a str) -> String;
    fn decrypt_journal_entry<'a>(password: &'a str, entry: &'a str) -> String;
}

pub struct ZeroSecurity;

impl Encryptor for ZeroSecurity {
    fn hash_password(password: &str) -> String {
        password.into()
    }
    fn verify_password<'a>(hashed_password: &'a str, entered_password: &'a str) -> bool {
        hashed_password == entered_password
    }
    fn encrypt_journal_entry<'a>(_password: &'a str, entry: &'a str) -> String {
        entry.into()
    }
    fn decrypt_journal_entry<'a>(_password: &'a str, entry: &'a str) -> String {
        entry.into()
    }
}
