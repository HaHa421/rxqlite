
use super::*;

use ::rsa::{RsaPublicKey, RsaPrivateKey,/* traits::PaddingScheme,*/ pkcs1v15::Pkcs1v15Encrypt};
use ::rsa::pkcs8::DecodePrivateKey;
use rand::rngs::OsRng;


pub struct RsaEncryptor {
    public_key: RsaPublicKey,
    private_key: RsaPrivateKey,
}

impl RsaEncryptor {
    pub fn new(pkcs8_key_der: &rustls::pki_types::PrivatePkcs8KeyDer) -> Self {
        let private_key = RsaPrivateKey::from_pkcs8_der(pkcs8_key_der.secret_pkcs8_der()).unwrap();
        let public_key = RsaPublicKey::from(&private_key);

        RsaEncryptor { public_key, private_key }
    }
}

impl EncryptData for RsaEncryptor {
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
        let padding = Pkcs1v15Encrypt;
        let mut rng = OsRng;
        let encrypted_data = self.public_key.encrypt(&mut rng, padding, &data[..]).map_err(|err|StorageIOError::write_logs(&err))?;
        Ok(encrypted_data)
    }

    fn decrypt(&self,data: Option<Vec<u8>>) -> Result<Option<Vec<u8>>,StorageIOError> {
      match data {
        None=>Ok(None),
        Some(data)=>{
          let padding = Pkcs1v15Encrypt;
          let decrypted_data = self.private_key.decrypt(padding, &data[..]).map_err(|err| StorageIOError::read_logs(&err))?;
          Ok(Some(decrypted_data))
        }
      }
    }
}

