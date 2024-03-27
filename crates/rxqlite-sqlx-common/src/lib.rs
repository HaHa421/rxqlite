#![deny(unused_extern_crates)]
#![deny(warnings)]

#[cfg(feature = "sqlite")]
use sqlx_sqlite_cipher::Sqlite;
#[cfg(feature = "sqlite")]
pub type SqlxDb = Sqlite;

use sqlx::prelude::*;
use sqlx::{database::HasArguments, Column, Database, Pool, TypeInfo};
use sqlx_core::types::chrono::{DateTime, Utc};

use rxqlite_common::{Message, MessageResponse, Value};
use sqlparser::ast::{Query, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

fn prepare_query<'q, DB: Database>(
    sql: &'q str,
    params: Vec<Value>,
) -> Result<sqlx::query::Query<'q, DB, <DB as HasArguments<'q>>::Arguments>, String>
where
    i64: Encode<'q, DB> + Type<DB>,
    &'q str: Encode<'q, DB> + Type<DB>,
    bool: Encode<'q, DB> + Type<DB>,
    String: Encode<'q, DB> + Type<DB>,
    f32: Encode<'q, DB> + Type<DB>,
    f64: Encode<'q, DB> + Type<DB>,
    DateTime<Utc>: Encode<'q, DB> + Type<DB>,
    Vec<u8>: Encode<'q, DB> + Type<DB>,
{
    let mut query = sqlx::query(sql);
    for param in params {
        match param {
            Value::Null => {
                return Err("passing null parameter considered an error".into());
            }
            Value::Bool(b) => {
                query = query.bind(b);
            }
            Value::Int(i) => {
                query = query.bind(i);
            }
            Value::F32(f) => {
                query = query.bind(f);
            }
            Value::F64(f) => {
                query = query.bind(f);
            }
            Value::String(s) => {
                query = query.bind(s);
            }
            Value::DateTime(dt) => {
                query = query.bind(dt);
            }
            Value::Blob(blob) => {
                query = query.bind(blob);
            }
        }
    }
    Ok(query)
}

pub async fn do_sql(pool: &Pool<SqlxDb>, message: Message) -> MessageResponse {
    match message {
        Message::Execute(sql, params) => {
            let query = prepare_query(&sql, params);
            if let Err(err) = &query {
                let response_message = MessageResponse::Error(format!("{}", err));
                return response_message;
            }
            let query = query.unwrap();
            let res = query.execute(pool).await;
            match res {
                Ok(_) => {
                    let response_message = MessageResponse::Rows(rxqlite_common::Rows::default());
                    response_message
                }
                Err(err) => {
                    let response_message = MessageResponse::Error(format!("{}", err));
                    response_message
                }
            }
        }
        Message::Fetch(sql, params) => {
            let query = prepare_query(&sql, params);
            if let Err(err) = &query {
                let response_message = MessageResponse::Error(format!("{}", err));
                return response_message;
            }
            let query = query.unwrap();
            let res = query.fetch_all(pool).await;
            let mut resulting_rows: Vec<rxqlite_common::Row> = vec![];
            match res {
                Ok(rows) => {
                    for row in rows.iter() {
                        let mut resulting_row: Vec<rxqlite_common::Col> = vec![];
                        let cols = row.len();
                        for i in 0..cols {
                            let col = row.column(i);
                            let type_info = col.type_info();
                            if type_info.is_null() {
                                resulting_row.push(Value::Null);
                            } else {
                                //println!("TYPE: {}",type_info.name());
                                match type_info.name() {
                                    "BOOL" => {
                                        let col: bool = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "INT" | "INTEGER" => {
                                        let col: i64 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "TEXT" => {
                                        let col: String = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "VARCHAR" => {
                                        let col: String = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "FLOAT" => {
                                        let col: f32 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "REAL" => {
                                        let col: f64 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "DATETIME" => {
                                        let col: DateTime<Utc> = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        resulting_rows.push(resulting_row.into());
                    }
                    let response_message = MessageResponse::Rows(resulting_rows);
                    response_message
                }
                Err(err) => {
                    let response_message = MessageResponse::Error(format!("{}", err));
                    response_message
                }
            }
        }
        Message::FetchOne(sql, params) => {
            let query = prepare_query(&sql, params);
            if let Err(err) = &query {
                let response_message = MessageResponse::Error(format!("{}", err));
                return response_message;
            }
            let query = query.unwrap();
            let res = query.fetch_one(pool).await;
            match res {
                Ok(row) => {
                    let mut resulting_row: Vec<rxqlite_common::Col> = vec![];
                    let cols = row.len();
                    for i in 0..cols {
                        let col = row.column(i);
                        let type_info = col.type_info();
                        if type_info.is_null() {
                            resulting_row.push(Value::Null);
                        } else {
                            //println!("TYPE: {}",type_info.name());
                            match type_info.name() {
                                "BOOL" => {
                                    let col: bool = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "INT" | "INTEGER" => {
                                    let col: i64 = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "TEXT" => {
                                    let col: String = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "VARCHAR" => {
                                    let col: String = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "FLOAT" => {
                                    let col: f32 = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "REAL" => {
                                    let col: f64 = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                "DATETIME" => {
                                    let col: DateTime<Utc> = row.get(i);
                                    resulting_row.push(col.into());
                                }
                                _ => {}
                            }
                        }
                    }
                    let response_message = MessageResponse::Rows(vec![resulting_row.into()]);
                    response_message
                }
                Err(err) => {
                    let response_message = MessageResponse::Error(format!("{}", err));
                    response_message
                }
            }
        }
        Message::FetchOptional(sql, params) => {
            let query = prepare_query(&sql, params);
            if let Err(err) = &query {
                let response_message = MessageResponse::Error(format!("{}", err));
                return response_message;
            }
            let query = query.unwrap();
            let res = query.fetch_optional(pool).await;
            match res {
                Ok(row) => {
                    if let Some(row) = row {
                        let mut resulting_row: Vec<rxqlite_common::Col> = vec![];
                        let cols = row.len();
                        for i in 0..cols {
                            let col = row.column(i);
                            let type_info = col.type_info();
                            if type_info.is_null() {
                                resulting_row.push(Value::Null);
                            } else {
                                //println!("TYPE: {}",type_info.name());
                                match type_info.name() {
                                    "BOOL" => {
                                        let col: bool = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "INT" | "INTEGER" => {
                                        let col: i64 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "TEXT" => {
                                        let col: String = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "VARCHAR" => {
                                        let col: String = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "FLOAT" => {
                                        let col: f32 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "REAL" => {
                                        let col: f64 = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    "DATETIME" => {
                                        let col: DateTime<Utc> = row.get(i);
                                        resulting_row.push(col.into());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        let response_message = MessageResponse::Rows(vec![resulting_row.into()]);
                        response_message
                    } else {
                        let response_message = MessageResponse::Rows(Default::default());
                        response_message
                    }
                }
                Err(err) => {
                    let response_message = MessageResponse::Error(format!("{}", err));
                    response_message
                }
            }
        }
    }
}

fn is_for_update_or_share(query: &Box<Query>) -> bool {
    !query.locks.is_empty()
}

pub fn is_query_write(sql: &str) -> bool {
    let ast = Parser::parse_sql(&GenericDialect, sql).unwrap();
    for stmt in ast {
        match stmt {
            Statement::Query(query) => {
                if is_for_update_or_share(&query) {
                    return true;
                } else {
                }
            }
            Statement::Insert { .. } | Statement::Update { .. } | Statement::Delete { .. } => {
                return true
            }
            _ => {
                return true;
            }
        }
    }
    false
}
