use crate::error::RXQLiteError;
use crate::type_info::DataType;
use crate::RXQLiteColumn;
use crate::{
    RXQLite, RXQLiteConnection, RXQLiteQueryResult, RXQLiteRow, RXQLiteStatement, RXQLiteTypeInfo,
    RXQLiteValue,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
//use futures_util::TryStreamExt;
use sqlx_core::describe::Describe;
use sqlx_core::error::Error;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::try_stream;
use sqlx_core::Either;

impl<'c> Executor<'c> for &'c mut RXQLiteConnection {
    type Database = RXQLite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<RXQLiteQueryResult, RXQLiteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

        Box::pin(try_stream! {
          let rows = self.inner.fetch_all(sql, match arguments {
            Some(arguments)=>arguments.values,
            _=>vec![],
          }).await;
          match rows {
            Ok(rows)=> {
              //println!("{}:({})",file!(),line!());

              //pin_mut!(cursor);
              let mut rows_iter=rows.into_iter();
              while let Some(row) = rows_iter.next() {
                let size = row.inner.len();
                let mut values = Vec::with_capacity(size);
                let mut columns = Vec::with_capacity(size);
                //let mut column_names = Vec::with_capacity(size);
                  for (i,value) in row.inner.into_iter().enumerate() {
                    values.push(RXQLiteValue::new(value,RXQLiteTypeInfo(DataType::Null)));
                    columns.push(RXQLiteColumn{
                      name : UStr::from(""),
                      ordinal: i,
                      type_info: RXQLiteTypeInfo(DataType::Null),
                    });
                  }
                  let row=Either::Right(RXQLiteRow {
                    values: values.into_boxed_slice(),
                    columns: columns.into(),
                    column_names : Default::default(),
                  });
                  r#yield!(row);
              }
              Ok(())
            }
            Err(err)=> {
              Err(RXQLiteError{
                inner: err,
              }.into())
            }
          }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<RXQLiteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());
        
          Box::pin( async {
          let row = self.inner.fetch_optional(sql, match arguments {
            Some(arguments)=>arguments.values,
            _=>vec![],
          }).await;
          match row {
            Ok(row)=> {
              //println!("{}:({})",file!(),line!());

              //pin_mut!(cursor);
              
              if let Some(row) = row {
                let size = row.inner.len();
                let mut values = Vec::with_capacity(size);
                let mut columns = Vec::with_capacity(size);
                //let mut column_names = Vec::with_capacity(size);
                  for (i,value) in row.inner.into_iter().enumerate() {
                    values.push(RXQLiteValue::new(value,RXQLiteTypeInfo(DataType::Null)));
                    columns.push(RXQLiteColumn{
                      name : UStr::from(""),
                      ordinal: i,
                      type_info: RXQLiteTypeInfo(DataType::Null),
                    });
                  }
                  let row=RXQLiteRow {
                    values: values.into_boxed_slice(),
                    columns: columns.into(),
                    column_names : Default::default(),
                  };
                  Ok(Some(row))
              } else {
                Ok(None)
              }
              
            }
            Err(err)=> {
              Err(RXQLiteError{
                inner: err,
              }.into())
            }
          }
          
          })
        
    }
    
    fn fetch_one<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<RXQLiteRow, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();
        //let persistent = query.persistent() && arguments.is_some();

        //let args = Vec::with_capacity(arguments.len());

          Box::pin( async {
          let row = self.inner.fetch_one(sql, match arguments {
            Some(arguments)=>arguments.values,
            _=>vec![],
          }).await;
          match row {
            Ok(row)=> {
              let size = row.inner.len();
              let mut values = Vec::with_capacity(size);
              let mut columns = Vec::with_capacity(size);
              //let mut column_names = Vec::with_capacity(size);
              for (i,value) in row.inner.into_iter().enumerate() {
                values.push(RXQLiteValue::new(value,RXQLiteTypeInfo(DataType::Null)));
                columns.push(RXQLiteColumn{
                  name : UStr::from(""),
                  ordinal: i,
                  type_info: RXQLiteTypeInfo(DataType::Null),
                });
              }
              let row=RXQLiteRow {
                values: values.into_boxed_slice(),
                columns: columns.into(),
                column_names : Default::default(),
              };
              Ok(row)
            }
            Err(err)=> {
              Err(RXQLiteError{
                inner: err,
              }.into())
            }
          }
          })
    }
    
    fn prepare_with<'e, 'q: 'e>(
        self,
        _sql: &'q str,
        _parameters: &[RXQLiteTypeInfo],
    ) -> BoxFuture<'e, Result<RXQLiteStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "prepare_with not supported",
            )))
        })
    }
    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, _sql: &'q str) -> BoxFuture<'e, Result<Describe<RXQLite>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "describe not supported",
            )))
        })
    }
}
