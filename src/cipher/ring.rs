use super::*;

use ::ring::aead::{AES_256_GCM, Nonce, Aad, /*BoundKey,*/ UnboundKey, LessSafeKey};
use ::ring::rand::{SystemRandom, SecureRandom};
use ::ring::pbkdf2;
use rustls::pki_types::PrivatePkcs8KeyDer;
use std::num::NonZeroU32;

pub struct Aes256GcmEncryptor {
    less_safe_key: LessSafeKey,
}

impl Aes256GcmEncryptor {
    pub fn new(pkcs8_key_der: &PrivatePkcs8KeyDer) -> Self {
        let key_bytes = pkcs8_key_der.secret_pkcs8_der();
        let mut derived_key = vec![0u8; AES_256_GCM.key_len()];
        let iterations = NonZeroU32::new(1).unwrap(); // only one iteration: private key is said secure
        static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
        pbkdf2::derive(
          PBKDF2_ALG,
          iterations, 
          key_bytes, // use key_bytes as salt
          &key_bytes,
          &mut derived_key,
      );

        let unbound_key = UnboundKey::new(&AES_256_GCM, &derived_key).unwrap();
        let less_safe_key = LessSafeKey::new(unbound_key);
        
        Aes256GcmEncryptor { less_safe_key }
    }
}

impl EncryptData for Aes256GcmEncryptor {
    fn encrypt(&self, mut data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
        let rng = SystemRandom::new();
        let mut nonce_ = [0u8; 12];
        rng.fill(&mut nonce_).map_err(|err|StorageIOError::write_logs(&std::io::Error::new(
          std::io::ErrorKind::Other,format!("{}",err).as_str()
        )))?;
        let nonce = Nonce::assume_unique_for_key(nonce_.clone());
        self.less_safe_key.seal_in_place_append_tag(nonce, Aad::empty(), &mut data).map_err(|err|StorageIOError::write_logs(&std::io::Error::new(
          std::io::ErrorKind::Other,format!("{}",err).as_str()
        )))?;
        
        let mut encrypted_data = nonce_.to_vec();
        encrypted_data.append(&mut data);
        Ok(encrypted_data)
    }

    fn decrypt(&self,data: Option<Vec<u8>>) -> Result<Option<Vec<u8>>,StorageIOError> {
      match data {
        None=>Ok(None),
        Some(data)=>{
          let nonce = &data[..12];
          let mut decrypted_data = data[12..].to_vec();
          
          let nonce = Nonce::assume_unique_for_key(nonce.try_into()
            .map_err(|err| StorageIOError::read_logs(&err))?
          );
          let decrypted_data = self.less_safe_key.open_in_place(nonce, Aad::empty(), &mut decrypted_data).map_err(|err|StorageIOError::read_logs(&std::io::Error::new(
          std::io::ErrorKind::Other,format!("{}",err).as_str()
        )))?;
    
          /*
          let decrypted_data = self.private_key.decrypt(padding, &data[..]).map_err(|err| StorageIOError::read_logs(&err))?;
          */
          Ok(Some(decrypted_data.to_vec()))
        }
      }
    }
}

