use crate::NodeId;
use std::sync::Arc;

type StorageIOError = openraft::StorageIOError<NodeId>;

pub trait EncryptData: Send + Sync + 'static {
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, StorageIOError>;
    fn decrypt(&self, data: &mut Vec<u8>) -> Result<(), StorageIOError>;
}

pub struct NoEncrypt;

impl EncryptData for NoEncrypt {
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, StorageIOError> {
        Ok(data)
    }
    fn decrypt(&self, _data: &mut Vec<u8>) -> Result<(), StorageIOError> {
        Ok(())
    }
}

impl EncryptData for Option<Arc<Box<dyn EncryptData>>> {
    fn encrypt(&self, data: Vec<u8>) -> Result<Vec<u8>, StorageIOError> {
        match self {
            Some(encrypt_data) => encrypt_data.encrypt(data),
            None => Ok(data),
        }
    }
    fn decrypt(&self, data: &mut Vec<u8>) -> Result<(), StorageIOError> {
        match self {
            Some(encrypt_data) => encrypt_data.decrypt(data),
            None => Ok(()),
        }
    }
}

//#[cfg(feature = "rsa-crate")]
//pub mod rsa;

//#[cfg(feature = "sqlcipher")]
//pub mod ring;

#[cfg(feature = "sqlcipher")]
pub mod aes_gcm_siv;
