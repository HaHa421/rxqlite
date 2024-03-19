
//use std::borrow::Cow;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::type_info::DataType;
use crate::types::Type;
use crate::{RXQLite, /*RaftSqliteArgumentValue, */ RaftSqliteTypeInfo, RaftSqliteValueRef};

impl Type<RXQLite> for str {
    fn type_info() -> RaftSqliteTypeInfo {
        RaftSqliteTypeInfo(DataType::Text)
    }
}

impl<'q> Encode<'q, RXQLite> for &'q str {
    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }
}
/*
impl<'r> Decode<'r, RXQLite> for &'r str {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text().map(|x| {
          x.as_str()
        })
    }
}
*/
impl Type<RXQLite> for Box<str> {
    fn type_info() -> RaftSqliteTypeInfo {
        <&str as Type<RXQLite>>::type_info()
    }
}

impl Encode<'_, RXQLite> for Box<str> {
    fn encode(self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }

    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }
}

impl Decode<'_, RXQLite> for Box<str> {
    fn decode(value: RaftSqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        value.text().map(Box::from)
    }
}

impl Type<RXQLite> for String {
    fn type_info() -> RaftSqliteTypeInfo {
        <&str as Type<RXQLite>>::type_info()
    }
}

impl<'q> Encode<'q, RXQLite> for String {
    fn encode(self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }

    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for String {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text()
    }
}
/*
impl Type<RXQLite> for Cow<'_, str> {
    fn type_info() -> RaftSqliteTypeInfo {
        <&str as Type<RXQLite>>::type_info()
    }

    fn compatible(ty: &RaftSqliteTypeInfo) -> bool {
        <&str as Type<RXQLite>>::compatible(ty)
    }
}

impl<'q> Encode<'q, RXQLite> for Cow<'q, str> {
    fn encode(self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }

    fn encode_by_ref(&self, args: &mut Vec<rxqlite::Value>) -> IsNull {
        args.push(rxqlite::Value::String(self.to_string()));

        IsNull::No
    }
}

impl<'r> Decode<'r, RXQLite> for Cow<'r, str> {
    fn decode(value: RaftSqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        value.text().map(Cow::Borrowed)
    }
}
*/
