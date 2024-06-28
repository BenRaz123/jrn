pub trait Encryptor {
    fn hash_password(password: &str) -> String;
    fn verify_password<'a>(hashed_password: &'a str, entered_password: &'a str) -> bool;
    fn encrypt_journal_entry<'a>(password: &'a str, entry: &'a str) -> String;
}
