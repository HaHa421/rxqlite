<h1 align="center">SQLx rqlite</h1>
<div align="center">
 <strong>
   The Rust SQL Toolkit rqlite driver
 </strong>
</div>
<br />

<div align="center">
  <h4>
    <a href="#install">
      Install
    </a>
    <span> | </span>
    <a href="#usage">
      Usage
    </a>
    <span> | </span>
    <a href="#security">
      Security
    </a>
    <span> | </span>
    <a href="#license">
      License
    </a>
  </h4>
</div>

<div align="center">
  <normal><a href="https://github.com/launchbadge/sqlx">Sqlx </a> driver for <a href="https://rqlite.io">rqlite</a></normal>
</div>

<br />

<div align="center">
  
  <!-- Version -->
  <a href="https://crates.io/crates/sqlx-rqlite">
    <img src="https://img.shields.io/crates/v/sqlx-rqlite.svg?style=flat-square"
    alt="Crates.io version" /></a>
  
</div>

## Install

You need to have access to an rqlite node.
Instructions for installing rqlite are available at https://rqlite.io


## Usage

A simple Cargo dependency would look like this :

```toml
[dependencies]
sqlx-rqlite = { version = "0.1" }
sqlx = {  version = "0.7" , default-features = false, features = ["macros", "runtime-tokio", "tls-none"] }
tokio = { version = "1", features = [ "full" ] }
```

Assuming an rqlite node listens at "127.0.0.1:4001", a simple app would proceed as follows:

```rust
use futures_util::StreamExt;
use sqlx::prelude::*;
use sqlx_rqlite::RqlitePoolOptions;

//#[async_std::main] // Requires the `attributes` feature of `async-std`
#[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
  let pool = RqlitePoolOptions::new()
        //.max_connections(5)
        .connect("rqlite://localhost:4001")
        .await?;
  sqlx::query(
        "CREATE TABLE IF NOT EXISTS _sqlx_rqlite_test_user_ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE
    )",
    )
    .execute(&pool)
    .await?;
    
  
    
  let mut row = sqlx::query("SELECT * FROM _sqlx_rqlite_test_user_  WHERE name = ?")
        .bind("JohnDoe")
        .fetch_optional(&pool)
        .await?;

    if row.is_none() {
        sqlx::query("INSERT INTO _sqlx_rqlite_test_user_  (name) VALUES (?);")
            .bind("JohnDoe")
            .execute(&pool)
            .await?;
        row = sqlx::query("SELECT * FROM _sqlx_rqlite_test_user_  WHERE name = 'JohnDoe'")
            .fetch_optional(&pool)
            .await?;
    }
    assert!(row.is_some());
    sqlx::query(
        "DROP TABLE _sqlx_rqlite_test_user_",
    )
    .execute(&pool)
    .await?;
    Ok(())
}
```

<br />

To get "datetime" support, you need to enable the feature "chrono".

<br />

## Security

For a secured connection, use:

```rust
let pool = RqlitePoolOptions::new()
        //.max_connections(5)
        .connect("rqlite://localhost:4001&ssl=yes")
        .await?;
```

?? **DANGER** In case you opt in for an insecure ssl connection 
(which accepts invalid certificates), use:

```rust
let pool = RqlitePoolOptions::new()
        //.max_connections(5)
        .connect("rqlite://localhost:4001&ssl-insecure=yes")
        .await?;
```

<br />

## License

Licensed under

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.