use sodiumoxide::crypto::aead::KEYBYTES;
use sodiumoxide::crypto::secretbox;
use sodiumoxide::crypto::pwhash;

use secretbox::NONCEBYTES;
use pwhash::SALTBYTES;

use serde::{Serialize, Deserialize};

const NONCESALTBYTES: usize = NONCEBYTES + SALTBYTES;

#[derive(Debug, PartialEq)]
pub enum CryptoError {
    DecryptionError,
}

#[derive(Serialize, Deserialize)]
pub struct CryptoModule {
    key: secretbox::Key,
    nonce: secretbox::Nonce,
    salt: pwhash::Salt,
}

impl CryptoModule {
    pub fn new(pass: &[u8]) -> CryptoModule {
        sodiumoxide::init().unwrap();

        let salt = pwhash::gen_salt();
        let mut k = secretbox::gen_key();
        let secretbox::Key(ref mut k) = k;

        let mut key_arr0 = [0; KEYBYTES];
        let key_arr1 = pwhash::derive_key(k, pass, &salt, pwhash::OPSLIMIT_INTERACTIVE, pwhash::MEMLIMIT_INTERACTIVE).unwrap();
        key_arr0.copy_from_slice(&key_arr1);
        let key = secretbox::Key(key_arr0);

        CryptoModule {
            key,
            nonce: secretbox::gen_nonce(),
            salt,
        }
    }

    pub fn import(pass: &[u8], byte_array: [u8; NONCESALTBYTES]) -> CryptoModule {
        sodiumoxide::init().unwrap();

        let mut salt_array = [0; SALTBYTES];
        let mut nonce_array = [0; NONCEBYTES];

        salt_array.copy_from_slice(&byte_array[..SALTBYTES]);
        nonce_array.copy_from_slice(&byte_array[SALTBYTES..]);

        let salt = pwhash::Salt(salt_array);
        let nonce = secretbox::Nonce(nonce_array);

        let mut k = secretbox::gen_key();
        let secretbox::Key(ref mut k) = k;

        let mut key_arr0 = [0; KEYBYTES];
        let key_arr1 = pwhash::derive_key(k, pass, &salt, pwhash::OPSLIMIT_INTERACTIVE, pwhash::MEMLIMIT_INTERACTIVE).unwrap();
        key_arr0.copy_from_slice(&key_arr1);
        let key = secretbox::Key(key_arr0);

        CryptoModule {
            key,
            nonce,
            salt,
        }
    }

    pub fn export(&self) -> [u8; NONCESALTBYTES] {
        let mut salt_nonce = [0; NONCESALTBYTES];
        salt_nonce[..SALTBYTES].copy_from_slice(&self.salt.0);
        salt_nonce[SALTBYTES..].copy_from_slice(&self.nonce.0);

        salt_nonce
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        secretbox::seal(plaintext, &self.nonce, &self.key)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        match secretbox::open(ciphertext, &self.nonce, &self.key) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => Err(CryptoError::DecryptionError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_setup() {
        let key = secretbox::gen_key();
        let nonce = secretbox::gen_nonce();
        // println!("key: {:?}\nnonce: {:?}\n", key.0, nonce);

        let plaintext = b"some data";
        let ciphertext = secretbox::seal(plaintext, &nonce, &key);
        let their_plaintext = secretbox::open(&ciphertext, &nonce, &key).unwrap();
        // println!("plaintext: {:?}\nciphertext: {:?}\ntheir_plaintext: {:?}", plaintext, ciphertext, their_plaintext);

        assert_eq!(plaintext, &their_plaintext[..])
    }

    #[test]
    fn happy_path() {
        let pass = "password12345";
        let pass_u8 = pass.as_bytes();

        let crypto1 = CryptoModule::new(pass_u8);
        let plaintext = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
        let ciphertext = crypto1.encrypt(plaintext);

        let byte_array = crypto1.export();

        let crypto2 = CryptoModule::import(pass_u8, byte_array);
        let plaintext2 = crypto2.decrypt(&ciphertext).unwrap_or(Vec::new());


        assert_eq!(plaintext, &plaintext2[..])
    }

    #[test]
    fn wrong_password() {
        let pass = "password12345";
        let pass_u8 = pass.as_bytes();

        let pass2 = "password54321";
        let pass2_u8 = pass2.as_bytes();

        let plaintext = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

        let crypto1 = CryptoModule::new(pass_u8);
        let ciphertext = crypto1.encrypt(plaintext);

        let byte_array = crypto1.export();

        let crypto2 = CryptoModule::import(pass2_u8, byte_array);
        let plaintext2 = crypto2.decrypt(&ciphertext);
        let err = plaintext2.expect_err("Decryption should fail");

        assert_eq!(err, CryptoError::DecryptionError)
    }

    #[test]
    fn key_conversion() {
        let key = secretbox::gen_key();
        let key_array = key.0;

        assert_eq!(key, secretbox::Key(key_array));
    }

    #[test]
    fn nonce_conversion() {
        let nonce = secretbox::gen_nonce();
        let nonce_array = nonce.0;

        assert_eq!(nonce, secretbox::Nonce(nonce_array));
    }
}