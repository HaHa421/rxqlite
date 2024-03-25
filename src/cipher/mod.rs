use std::sync::Arc;
use crate::NodeId;
use std::ops::Range;

type StorageIOError = openraft::StorageIOError<NodeId>;

pub trait EncryptData : Send + Sync + 'static { 
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError>;
  fn decrypt(&self,data: &mut [u8]) -> Result<Range<usize>,StorageIOError>;
}

pub struct NoEncrypt;

impl EncryptData for NoEncrypt {
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
    Ok(data)
  }
  fn decrypt(&self,data: &mut [u8]) -> Result<Range<usize>,StorageIOError> {
    Ok(0..data.len())
  }
}

impl EncryptData for Option<Arc<Box<dyn EncryptData>>> {
  fn encrypt(&self,data: Vec<u8>) -> Result<Vec<u8>,StorageIOError> {
    match self {
      Some(encrypt_data)=>encrypt_data.encrypt(data),
      None=>Ok(data),
    }
  }
  fn decrypt(&self,data: &mut [u8]) -> Result<Range<usize>,StorageIOError> {
    match self {
      Some(encrypt_data)=>encrypt_data.decrypt(data),
      None=>Ok(0..data.len()),
    }
  }
}

//#[cfg(feature = "rsa-crate")]
//pub mod rsa;

#[cfg(feature = "sqlcipher")]
pub mod ring;



