//! module for interacting with application state, and writing to/reading from a JSON file

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use base64::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{
    date::Date,
    encryptor::{DecryptError, Encryptor},
};

#[derive(Debug, Clone)]
pub struct EncryptedJournal {
    pub password_hash: String,
    pub kdf_salt: [u8; 32],
    pub entries: HashSet<EncryptedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An encrypted journal, use for storing in a secure manner
pub struct StoredJournal {
    /// Hash of the password
    pub password_hash: String,
    /// Salt for kdf (key is reused)
    pub kdf_salt: String,
    /// Set of [entries](`StoredEntry`)
    pub entries: HashSet<StoredEntry>,
}

#[derive(Hash, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// An entry represented in storage
pub struct EncryptedEntry {
    /// The date. Not enrypted
    pub date: Date,
    /// The nonce used for encryption (should be 12 bytes)
    pub nonce: [u8; 12],
    /// Encrypted digest of journal entry
    pub digest: Vec<u8>,
}
#[derive(Hash, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StoredEntry {
    pub date: Date,
    pub nonce: String,
    pub digest: String,
}
#[derive(Debug)]
pub enum FromBase64Error {
    NotValidBase64,
    InvalidLength,
}

impl TryFrom<StoredJournal> for EncryptedJournal {
    type Error = FromBase64Error;
    fn try_from(value: StoredJournal) -> Result<Self, Self::Error> {
        let password_hash = value.password_hash;
        let kdf_salt = try_b64_to_arr(&value.kdf_salt)?;
        let mut entries = HashSet::new();
        for entry in value.entries {
            entries.insert(entry.try_into()?);
        }
        Ok(Self {
            password_hash,
            kdf_salt,
            entries,
        })
    }
}

impl From<EncryptedJournal> for StoredJournal {
    fn from(value: EncryptedJournal) -> Self {
        let password_hash = value.password_hash;
        let kdf_salt = BASE64_STANDARD.encode(value.kdf_salt);
        let entries = value
            .entries
            .iter()
            .map(|entry| StoredEntry::from(entry.clone()))
            .collect();
        Self {
            password_hash,
            kdf_salt,
            entries,
        }
    }
}

impl TryFrom<StoredEntry> for EncryptedEntry {
    type Error = FromBase64Error;
    fn try_from(value: StoredEntry) -> Result<Self, Self::Error> {
        let date = value.date;
        let nonce = try_b64_to_arr(&value.nonce)?;
        let digest = try_b64_to_vec(&value.digest)?;
        Ok(Self {
            date,
            nonce,
            digest,
        })
    }
}

impl From<EncryptedEntry> for StoredEntry {
    fn from(value: EncryptedEntry) -> Self {
        let date = value.date;
        let nonce = BASE64_STANDARD.encode(value.nonce);
        let digest = BASE64_STANDARD.encode(value.digest);
        Self {
            date,
            nonce,
            digest,
        }
    }
}

fn try_b64_to_arr<const N: usize>(str: &str) -> Result<[u8; N], FromBase64Error> {
    let attempted_decoded = BASE64_STANDARD.decode(str);
    if attempted_decoded.is_err() {
        return Err(FromBase64Error::NotValidBase64);
    }
    let decoded = attempted_decoded.unwrap();
    match decoded.len() == N {
        true => Ok(decoded.try_into().unwrap()),
        false => Err(FromBase64Error::InvalidLength),
    }
}

fn try_b64_to_vec(str: &str) -> Result<Vec<u8>, FromBase64Error> {
    let attempted_decode = BASE64_STANDARD.decode(str);
    match attempted_decode {
        Ok(v) => Ok(v),
        Err(_) => Err(FromBase64Error::NotValidBase64),
    }
}

#[derive(Debug, Clone)]
/// A journal. contains a password, and a set of entries.
pub struct State {
    /// a password
    pub password: String,
    /// a set of entries
    pub entries: HashMap<Date, String>,
}

/// how loading, deserializing, and unencrypting a file could go wrong
#[derive(Debug)]
pub enum LoadError {
    /// the file either has inadequate permissions or does not exist
    NotAccessible,
    /// the file could not be parsed
    ParseError,
    FromBase64Error(FromBase64Error),
    /// the password given was incorrect and the file could not be unencrypted
    IncorrectPassword,
}

/// how encrypting, serializing, and writing to a file could go wrong
#[derive(Debug)]
pub enum SaveError {
    /// should never happen. for some reason, [`serde`] could not serialize
    SerializationError,
    /// file could not be written to
    FileError,
}

impl State {
    /// gets the journal entry at a given timestamp
    pub fn get_entry(&self, date: &Date) -> Option<String> {
        self.entries.get(date).cloned()
    }

    /// create or overide an entry at a given date
    pub fn set_entry(&mut self, date: &Date, content: &str) {
        self.entries.insert(date.clone(), content.into());
    }

    /// a convenience function for getting the value of today's entry
    pub fn get_today(&self) -> Option<String> {
        self.get_entry(&Date::today())
    }

    /// a convenience function for setting the value of today's entry
    pub fn set_today(&mut self, content: &str) {
        self.set_entry(&Date::today(), content);
    }

    /// initializes a journal
    pub fn new() -> Self {
        Self {
            password: "".into(),
            entries: HashMap::new(),
        }
    }

    /// changes password
    pub fn change_password(&mut self, new_password: &str) {
        self.password = new_password.into();
    }

    /// loads a file at the given name, deserializes the data, and unencrypts with the given
    /// password
    pub fn load<E: Encryptor>(
        &mut self,
        file_name: &str,
        password: &str,
        e: &E,
    ) -> Result<(), LoadError> {
        let json = fs::read(file_name);
        if json.is_err() {
            return Err(LoadError::NotAccessible);
        }
        let json = String::from_utf8(json.unwrap()).unwrap();

        let stored_journal = serde_json::from_str::<StoredJournal>(&json);

        if stored_journal.is_err() {
            return Err(LoadError::ParseError);
        }

        let stored_journal = stored_journal.unwrap();

        let encrypted_journal = EncryptedJournal::try_from(stored_journal);

        if let Err(e) = encrypted_journal {
            return Err(LoadError::FromBase64Error(e));
        }

        let encrypted_journal = encrypted_journal.unwrap();

        let state = e.decrypt_journal(&encrypted_journal, password);

        if let Err(DecryptError::IncorrectPassword) = state {
            return Err(LoadError::IncorrectPassword);
        }

        *self = state.unwrap();

        Ok(())
    }

    /// encrypts contents, serializes contents, and writes them to the given file
    pub fn save<E: Encryptor>(&self, file_name: &str, e: &E) -> Result<(), SaveError> {
        let encrypted_journal = e.encrypt_journal(self);

        let saved_journal: StoredJournal = encrypted_journal.into();

        let json = serde_json::to_string(&saved_journal);

        if json.is_err() {
            return Err(SaveError::SerializationError);
        }

        let json = json.unwrap();

        let err = fs::write(file_name, json);

        if err.is_err() {
            return Err(SaveError::FileError);
        }

        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
