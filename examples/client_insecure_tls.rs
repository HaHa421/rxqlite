#![deny(warnings)]

use rxqlite_common::{Message, MessageResponse, RSQliteClientTlsConfig};
use sqlx::types::chrono::{DateTime, Utc};

#[tokio::main]
async fn main() {
    let client = rxqlite::client::RXQLiteClientBuilder::new(1, "127.0.0.1:21001".into())
        .tls_config(Some(
            RSQliteClientTlsConfig::default().accept_invalid_certificates(true),
        ))
        .build();

    for _i in 0..1 {
        let metrics = client.metrics().await;
        println!("{:#?}", metrics);

        let message = Message::Execute(
            "CREATE TABLE IF NOT EXISTS _test_user_ (
          id INTEGER PRIMARY KEY,
          name TEXT NOT NULL UNIQUE,
          birth_date DATETIME NOT NULL
          )"
            .into(),
            vec![],
        );
        let response = client.sql(&message).await;
        println!("Server response: {:#?}", response);

        for i in 0..100 {
            let name = format!("ha-{}", i);
            let birth_date = Utc::now();
            let message = Message::Execute(
                "INSERT INTO _test_user_ (name,birth_date) VALUES (?,?)".into(),
                vec![name.into(), birth_date.into()],
            );
            let response = client.sql(&message).await;
            println!("Server answer: {:?}", response);
        }

        let message = Message::Fetch(
            "SELECT name,birth_date from _test_user_ where name = ?".into(),
            vec!["ha-10".into()],
        );
        let response = client.sql(&message).await;
        match response {
            Ok(response) => match response.data {
                Some(MessageResponse::Rows(rows)) => {
                    for row in &rows {
                        let name: String = row.get(0);
                        let birth_date: DateTime<Utc> = row.get(1);
                        println!(
                            r#"name : {}
                birth_date: {}
"#,
                            name, birth_date
                        );
                    }
                }
                Some(MessageResponse::Error(err)) => {
                    eprintln!("error : {}", err);
                }
                _ => {
                    eprintln!("no response");
                }
            },
            Err(err) => {
                eprintln!("error : {}", err);
            }
        }
        let message = Message::Fetch("SELECT * from _test_user_".into(), vec![]);
        let response = client.sql(&message).await;
        match response {
            Ok(response) => match response.data {
                Some(MessageResponse::Rows(rows)) => {
                    for row in &rows {
                        let id: i64 = row.get(0);
                        let name: String = row.get(1);
                        let birth_date: DateTime<Utc> = row.get(2);
                        println!(
                            r#"id: {}
name : {}
birth_date: {}
"#,
                            id, name, birth_date
                        );
                    }
                }
                Some(MessageResponse::Error(err)) => {
                    eprintln!("error : {}", err);
                }
                _ => {
                    eprintln!("no response");
                }
            },
            Err(err) => {
                eprintln!("error : {}", err);
            }
        }

        for i in 0..100 {
            let name = format!("ha-{}", i);
            let message = Message::Execute(
                "DELETE FROM _test_user_ where name = ?".into(),
                vec![name.into()],
            );
            let response = client.sql(&message).await;
            println!("Server answer: {:?}", response);
        }

        let message = Message::Execute("DROP TABLE _test_user_ ".into(), vec![]);
        let response = client.sql(&message).await;
        println!("Server answer: {:?}", response);
    }
}
