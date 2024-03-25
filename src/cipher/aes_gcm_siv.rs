use super::*;

use ::aes_gcm_siv::{Aes256GcmSiv,AeadInPlace,KeyInit,Nonce};
use ::aes_gcm_siv::aead::generic_array::GenericArray;


use ::ring::rand::{SystemRandom, SecureRandom};
use ::ring::pbkdf2;
use rustls::pki_types::PrivatePkcs8KeyDer;
use std::num::NonZeroU32;

pub struct Aes256GcmSivEncryptor {
    cipher: Aes256GcmSiv,
}

impl Aes256GcmSivEncryptor {
    pub fn new(pkcs8_key_der: &PrivatePkcs8KeyDer) -> Self {
        let key_bytes = pkcs8_key_der.secret_pkcs8_der();
        let mut derived_key = vec![0u8; 32];
        let iterations = NonZeroU32::new(1).unwrap(); // only one iteration: private key is said secure
        static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
        pbkdf2::derive(
          PBKDF2_ALG,
          iterations, 
          key_bytes, // use key_bytes as salt
          &key_bytes,
          &mut derived_key,
      );

        let key = GenericArray::clone_from_slice(&derived_key);
        let cipher = Aes256GcmSiv::new(&key);
        
        Aes256GcmSivEncryptor { cipher }
    }
}

impl EncryptData for Aes256GcmSivEncryptor {
    fn encrypt(&self, mut data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
        let rng = SystemRandom::new();
        let mut nonce_ = [0u8; 12];
        rng.fill(&mut nonce_).map_err(|err|StorageIOError::write_logs(&std::io::Error::new(
          std::io::ErrorKind::Other,format!("{}",err).as_str()
        )))?;
        let nonce = Nonce::from_slice(&nonce_);
        self.cipher.encrypt_in_place(nonce, b"", &mut data).map_err(|err|StorageIOError::write_logs(&std::io::Error::new(
          std::io::ErrorKind::Other,format!("{}",err).as_str()
        )))?;
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.append(&mut data);
        Ok(encrypted_data)
    }

    fn decrypt(&self, data: &mut Vec<u8>) -> Result<(), StorageIOError> {
      let nonce = &data[..12];
      let nonce: [u8 ; 12] = nonce.to_vec().try_into().unwrap();
      let nonce = Nonce::from(nonce);
      data.drain(0..12);
      
      match self.cipher.decrypt_in_place(&nonce, b"", data) {
          Ok(_) => {
              Ok(())
          },
          Err(err) => Err(StorageIOError::read_logs(&std::io::Error::new(
              std::io::ErrorKind::Other, format!("{}", err).as_str()
          ))),
      }
    }
}
