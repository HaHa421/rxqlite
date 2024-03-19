use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use sqlx::prelude::*;
use sqlx_rxqlite::RaftSqlitePoolOptions;

//#[async_std::main] // Requires the `attributes` feature of `async-std`
#[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let host = if std::env::args().len() > 1 {
      std::env::args().nth(1).unwrap().to_string()
    } else {
      "locahost".into()
    };
    // Create a connection pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let pool = RaftSqlitePoolOptions::new()
        //.max_connections(5)
        .connect(&format!("rqlite://{}:21001",host))
        .await?;
    println!("connected");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _sqlx_rqlite_test_user_and_date_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        birth_date DATETIME NOT NULL
    )",
    )
    .execute(&pool)
    .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let mut rows = sqlx::query("SELECT id,name,birth_date FROM _sqlx_rqlite_test_user_and_date_").fetch(&pool);
    println!("fetched rows");
    while let Some(row) = rows.next().await {
        println!("got row");
        let row = row?;
        //println!("row: {:?}",row);
        let id: i64 = row.get(0);
        let name: String = row.get(1);
        let birth_date: DateTime<Utc> = row.get(2);
        //println!("{} : {}", id,name/*, birth_date*/);
        println!("{} : {} | {:?}", id, name, birth_date);
    }
    let mut row = sqlx::query("SELECT * FROM _sqlx_rqlite_test_user_and_date_ WHERE name = ?")
        .bind("ha2")
        .fetch_optional(&pool)
        .await?;

    if row.is_none() {
        sqlx::query("INSERT INTO _sqlx_rqlite_test_user_and_date_ (name,birth_date) VALUES (?, ?);")
            .bind("ha2")
            .bind(Utc::now())
            .execute(&pool)
            .await?;
        row = sqlx::query("SELECT * FROM _sqlx_rqlite_test_user_and_date_ WHERE name = 'ha2'")
            .fetch_optional(&pool)
            .await?;
    }

    let row = row.expect("insertion failed");
    //println!("row: {:?}",row);
    let id: i64 = row.get(0);
    let idf64: f64 = row.get(0);
    let name: String = row.get(1);
    let birth_date: DateTime<Utc> = row.get(2);
    //println!("{} : {}", id,name/*, birth_date*/);
    println!("{}(as f64: {}) : {} | {:?}", id, idf64, name, birth_date);

    sqlx::query("DELETE FROM _sqlx_rqlite_test_user_and_date_ WHERE name = ?")
        .bind("ha2")
        .execute(&pool)
        .await?;
    
    sqlx::query("DROP _sqlx_rqlite_test_user_and_date_")
        .execute(&pool)
        .await?;
        
    println!("finishing");

    Ok(())
}
