use futures_util::StreamExt;
use sqlx::prelude::*;
use sqlx_rxqlite::RXQLitePoolOptions;

//#[async_std::main] // Requires the `attributes` feature of `async-std`
#[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    let host = if std::env::args().len() > 1 {
      std::env::args().nth(1).unwrap().to_string()
    } else {
      "localhost".into()
    };
    // Create a connection pool
    //  for MySQL/MariaDB, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    let pool = RXQLitePoolOptions::new()
        //.max_connections(5)
        .connect(&format!("rxqlite://{}:21001",host))
        .await?;
    println!("connected");
    sqlx::query("DROP TABLE IF EXISTS _sqlx_rxqlite_test_user_")
        .execute(&pool)
        .await?;
        
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _sqlx_rxqlite_test_user_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE
    )",
    )
    .execute(&pool)
    .await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
    let mut rows = sqlx::query("SELECT id,name FROM _sqlx_rxqlite_test_user_").fetch(&pool);
    println!("fetched rows");
    while let Some(row) = rows.next().await {
        println!("got row");
        let row = row?;
        //println!("row: {:?}",row);
        let id: i64 = row.get(0);
        let name: String = row.get(1);

        //println!("{} : {}", id,name/*, birth_date*/);
        println!("{} : {}", id, name);
    }
    let mut row = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_ WHERE name = ?")
        .bind("ha2")
        .fetch_optional(&pool)
        .await?;

    if row.is_none() {
        sqlx::query("INSERT INTO _sqlx_rxqlite_test_user_ (name) VALUES (?);")
            .bind("ha2")
            .execute(&pool)
            .await?;
        /*
        row = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_ WHERE name = 'ha2'")
            .fetch_optional(&pool)
            .await?;
         */
        row = sqlx::query("SELECT * FROM _sqlx_rxqlite_test_user_ WHERE name = ?")
        .bind("ha2")
        .fetch_optional(&pool)
        .await?;
    }

    let row = row.expect("insertion failed");
    //println!("row: {:?}",row);
    let id: i64 = row.get(0);
    let idf64: f64 = row.get(0);
    let name: String = row.get(1);
    //println!("{} : {}", id,name/*, birth_date*/);
    println!("{}(as f64: {}) : {}", id, idf64, name);

    sqlx::query("DELETE FROM _sqlx_rxqlite_test_user_ WHERE name = ?")
        .bind("ha2")
        .execute(&pool)
        .await?;
    
    sqlx::query("DROP TABLE _sqlx_rxqlite_test_user_")
        .execute(&pool)
        .await?;
        
    println!("finishing");

    Ok(())
}
