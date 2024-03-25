use std::sync::Arc;
use crate::NodeId;

type StorageIOError = openraft::StorageIOError<NodeId>;

pub trait EncryptData : Send + Sync + 'static { 
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError>;
  fn decrypt(&self,data: Option<Vec<u8>>) -> Result<Option<Vec<u8>>,StorageIOError>;
}

pub struct NoEncrypt;

impl EncryptData for NoEncrypt {
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
    Ok(data)
  }
  fn decrypt(&self,data: Option<Vec<u8>>) -> Result<Option<Vec<u8>>,StorageIOError> {
    Ok(data)
  }
}

impl EncryptData for Option<Arc<Box<dyn EncryptData>>> {
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
    match self {
      Some(encrypt_data)=>encrypt_data.encrypt(data),
      None=>Ok(data),
    }
  }
  fn decrypt(&self,data: Option<Vec<u8>>) -> Result<Option<Vec<u8>>,StorageIOError> {
    match self {
      Some(encrypt_data)=>encrypt_data.decrypt(data),
      None=>Ok(data),
    }
  }
}

#[cfg(feature = "rsa-crate")]
pub mod rsa;

#[cfg(feature = "sqlcipher")]
pub mod ring;



