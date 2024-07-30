//! module for the [`Encryptor`] trait. Contains [`ZeroSecurity`] and [`Secure`] Implementations.

use std::{
    collections::{HashMap, HashSet},
    process::exit,
};

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use bcrypt::DEFAULT_COST;
use pbkdf2::pbkdf2_hmac;
use rand::Rng;
use sha2::Sha256;

use crate::{
    date::Date,
    db::{EncryptedEntry, EncryptedJournal, State, StoredEntry, StoredJournal},
};

#[derive(Debug)]
/// the ways in which decrypting a [`StoredJournal`] can go wrong
pub enum DecryptError {
    /// the password was incorrect (see [`Encryptor::verify_password()`])
    IncorrectPassword,
}

/// A implementation-agnostic abstraction over methods for encrypting, decrypting, and hashing.
pub trait Encryptor {
    /// A 1->1 hash function
    fn hash_password<'a>(&self, password: &'a str) -> String;
    /// Verify password using hashed password
    fn verify_password<'a>(
        &self,
        hashed_password: &'a str,
        entered_password: &'a str,
    ) -> bool;
    /// Use password to encrypt a journal entry
    fn encrypt_journal_entry<'a>(
        &self,
        key: [u8; 32],
        entry: &'a str,
        date: &Date,
    ) -> EncryptedEntry;
    /// Use password to decrypt a journal entry
    fn decrypt_journal_entry<'a>(
        &self,
        key: [u8; 32],
        entry: &'a EncryptedEntry,
    ) -> (Date, String);
    fn make_kdf_salt(&self) -> [u8; 32];
    fn gen_key<'a>(&self, password: &'a str, kdf_salt: [u8; 32]) -> [u8; 32];
    /// Provided. Encrypt journal state.
    fn encrypt_journal<'a>(&self, journal: &'a State) -> EncryptedJournal {
        let password_hash = self.hash_password(&journal.password);
        let kdf_salt = self.make_kdf_salt();
        let key = self.gen_key(&journal.password, kdf_salt);

        let entries: HashSet<EncryptedEntry> = journal
            .entries
            .iter()
            .map(|(date, entry)| {
                self.encrypt_journal_entry(key, entry, date)
            })
            .collect();

        EncryptedJournal {
            password_hash,
            kdf_salt,
            entries,
        }
    }
    /// Provided. Decrypts stored journal into application state
    fn decrypt_journal<'a>(
        &self,
        encrypted_journal: &'a EncryptedJournal,
        password: &'a str,
    ) -> Result<State, DecryptError> {
        let password = password.to_string();

        if !(self.verify_password(&encrypted_journal.password_hash, &password))
        {
            return Err(DecryptError::IncorrectPassword);
        }

        let kdf_salt = encrypted_journal.kdf_salt;
        let key = self.gen_key(&password, kdf_salt);

        let entries: HashMap<Date, String> = encrypted_journal
            .entries
            .iter()
            .map(|entry| self.decrypt_journal_entry(key, entry))
            .collect();

        Ok(State { password, entries })
    }
}

/// Bare bones implementation that satisfies [`Encryptor`]. Does not employ any hashing or
/// encryption. **DO NOT USE IN PRODUCTION!**
pub struct ZeroSecurity;

impl Encryptor for ZeroSecurity {
    fn gen_key<'a>(&self, _password: &'a str, _kdf_salt: [u8; 32]) -> [u8; 32] {
        Default::default()
    }
    fn make_kdf_salt(&self) -> [u8; 32] {
        Default::default()
    }
    fn hash_password<'a>(&self, password: &'a str) -> String {
        password.into()
    }
    fn verify_password<'a>(
        &self,
        hashed_password: &'a str,
        entered_password: &'a str,
    ) -> bool {
        hashed_password == entered_password
    }
    fn encrypt_journal_entry<'a>(
        &self,
        _key: [u8; 32],
        entry: &'a str,
        date: &Date,
    ) -> EncryptedEntry {
        EncryptedEntry {
            date: date.clone(),
            nonce: Default::default(),
            digest: entry.bytes().collect(),
        }
    }
    fn decrypt_journal_entry<'a>(
        &self,
        _key: [u8; 32],
        entry: &'a EncryptedEntry,
    ) -> (Date, String) {
        (
            entry.date.clone(),
            String::from_utf8(entry.digest.clone()).unwrap(),
        )
    }
}

/// [`Encryptor`] implementation that uses
/// - [bcrypt](https://wikipedia.org/wiki/Bcrypt) for password hashing and verification
/// - [aes-gcm-siv](https://wikipedia.org/wiki/AES-GCM-SIV) for content encryption (256-bit
/// keylength)
///     - 96-bit nonce
/// - [pbkdf2](https://wikipedia.org/wiki/PBKDF2) for key derivation
///     - 256-bit salt
pub struct Secure;

impl Encryptor for Secure {
    fn hash_password<'a>(&self, password: &'a str) -> String {
        bcrypt::hash(password, DEFAULT_COST).unwrap()
    }
    fn verify_password<'a>(
        &self,
        hashed_password: &'a str,
        entered_password: &'a str,
    ) -> bool {
        bcrypt::verify(entered_password, hashed_password).unwrap()
    }
    fn gen_key<'a>(&self, password: &'a str, kdf_salt: [u8; 32]) -> [u8; 32] {
        println!(":: Gen Key...");
        let mut key: [u8; 32] = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            &kdf_salt,
            300_000,
            &mut key,
        );
        key
    }
    fn make_kdf_salt(&self) -> [u8; 32] {
        let mut rng = rand::thread_rng();
        rng.gen()
    }
    fn encrypt_journal_entry<'a>(
        &self,
        key: [u8; 32],
        entry: &'a str,
        date: &Date,
    ) -> EncryptedEntry {
        let mut rng = rand::thread_rng();
        let nonce: [u8; 12] = rng.gen();
        let digest = self.aes_encrypt(&key, &nonce, entry);

        EncryptedEntry {
            date: date.clone(),
            nonce,
            digest,
        }
    }
    fn decrypt_journal_entry<'a>(
        &self,
        key: [u8; 32],
        entry: &'a EncryptedEntry,
    ) -> (Date, String) {
        let EncryptedEntry {
            date,
            nonce,
            digest,
        } = entry;

        let cleartext = self.aes_decrypt(&key, nonce, digest.clone());
        (date.clone(), cleartext)
    }
}

impl Secure {
    fn aes_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        cleartext: &str,
    ) -> Vec<u8> {
        let cipher = Aes256GcmSiv::new_from_slice(key).expect("Key is 256 bit");
        let nonce = Nonce::from_slice(nonce);

        let ciphertext = cipher.encrypt(nonce, cleartext.as_bytes());
        ciphertext.unwrap()
    }
    fn aes_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        ciphertext: Vec<u8>,
    ) -> String {
        let cipher = Aes256GcmSiv::new_from_slice(key).unwrap();
        let nonce = Nonce::from_slice(nonce);

        let cleartext = cipher.decrypt(nonce, ciphertext.as_slice()).unwrap();

        String::from_utf8(cleartext).unwrap()
    }
}
