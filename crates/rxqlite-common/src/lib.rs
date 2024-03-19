#![deny(unused_extern_crates)]
#![deny(warnings)]

use serde::{Serialize, Deserialize};
use chrono::{DateTime,Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
  Null,
  Bool(bool),
  Int(i64),
  F32(f32),
  F64(f64),
  String(String),
  DateTime(DateTime<Utc>),
  Blob(Vec<u8>),
}
impl From<i64> for Value {
  fn from(i:i64)->Self {
    Self::Int(i)
  }
}
impl From<bool> for Value {
  fn from(b:bool)->Self {
    Self::Bool(b)
  }
}
impl From<f32> for Value {
  fn from(f:f32)->Self {
    Self::F32(f)
  }
}
impl From<f64> for Value {
  fn from(f:f64)->Self {
    Self::F64(f)
  }
}
impl From<String> for Value {
  fn from(s:String)->Self {
    Self::String(s)
  }
}

impl From<&str> for Value {
  fn from(s:&str)->Self {
    Self::String(s.into())
  }
}

impl From<DateTime<Utc>> for Value {
  fn from(dt:DateTime<Utc>)->Self {
    Self::DateTime(dt)
  }
}

pub trait FromValueRef {
  fn from_value_ref(value: &Value)->Self;
}

impl FromValueRef for i64 {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>0, 
      Value::Bool(b)=>if *b { 1 } else { 0 },
      Value::Int(i)=>*i,
      Value::F32(f)=>*f as _,
      Value::F64(f)=>*f as _,
      Value::String(s)=> i64::from_str_radix(&s,10).unwrap_or(0),
      Value::DateTime(dt)=> dt.timestamp() as _,
      Value::Blob(_)=>0,
    }
  }
}
impl FromValueRef for i32 {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>0, 
      Value::Bool(b)=>if *b { 1 } else { 0 },
      Value::Int(i)=>*i as _,
      Value::F32(f)=>*f as _,
      Value::F64(f)=>*f as _,
      Value::String(s)=> i32::from_str_radix(&s,10).unwrap_or(0),
      Value::DateTime(dt)=> dt.timestamp() as _,
      Value::Blob(_)=>0,
    }
  }
}
impl FromValueRef for bool {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>false,
      Value::Bool(b)=>*b,
      Value::Int(i)=>*i != 0,
      Value::F32(f)=>*f != 0.,
      Value::F64(f)=>*f != 0.,
      Value::String(s)=> s=="true" || s=="True" || s=="1",
      Value::DateTime(dt)=> dt.timestamp() != 0,
      Value::Blob(b)=> b.len() != 0,
    }
  }
}

impl FromValueRef for f32 {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>0.,
      Value::Bool(b)=> if *b { 1. } else { 0. },
      Value::Int(i)=>*i as _,
      Value::F32(f)=>*f,
      Value::F64(f)=>*f as _,
      Value::String(s)=> s.parse().unwrap_or(0.),
      Value::DateTime(dt)=> dt.timestamp() as _,
      Value::Blob(_)=> 0.,
    }
  }
}

impl FromValueRef for f64 {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>0.,
      Value::Bool(b)=> if *b { 1. } else { 0. },
      Value::Int(i)=>*i as _,
      Value::F32(f)=>*f as _,
      Value::F64(f)=>*f,
      Value::String(s)=> s.parse().unwrap_or(0.),
      Value::DateTime(dt)=> dt.timestamp() as _,
      Value::Blob(_)=> 0.,
    }
  }
}

impl FromValueRef for String {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::Null=>Default::default(),
      Value::Bool(b)=> if *b { "true".into() } else { "false".into() },
      Value::Int(i)=>i.to_string(),
      Value::F32(f)=>f.to_string(),
      Value::F64(f)=>f.to_string(),
      Value::String(s)=> s.clone(),
      Value::DateTime(dt)=> dt.to_rfc3339(),
      Value::Blob(_)=> Default::default(),
    }
  }
}

impl FromValueRef for DateTime<Utc> {
  fn from_value_ref(v:&Value)->Self {
    match v {
      Value::DateTime(dt)=> dt.clone(),
      _=> Default::default(),
    }
  }
}



pub type Col = Value;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Row {
  pub inner: Vec<Col>,
}

impl std::ops::Deref for Row {
  type Target = Vec<Col>;
  fn deref(&self)->&Self::Target {
    &self.inner
  }
}

impl std::ops::DerefMut for Row {
  fn deref_mut(&mut self)->&mut Self::Target {
    &mut self.inner
  }
}

impl Row {
  pub fn get<T: FromValueRef>(&self,idx : usize)->T {
    T::from_value_ref(&self[idx])
  }
}
impl From<Vec<Value>> for Row {
  fn from(inner: Vec<Value>)->Self {
    Self {
      inner,
    }
  }
}

pub type Rows = Vec<Row>;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Execute(String,Vec<Value>),
    Fetch(String,Vec<Value>),
    FetchOne(String,Vec<Value>),
    FetchOptional(String,Vec<Value>),
}

impl Message {
  pub fn sql(&self)->&str {
    match self {
      Self::Execute(s,_)=>s.as_str(),
      Self::Fetch(s,_)=>s.as_str(),
      Self::FetchOne(s,_)=>s.as_str(),
      Self::FetchOptional(s,_)=>s.as_str(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageResponse {
  Rows(Rows),
  Error(String),
}


