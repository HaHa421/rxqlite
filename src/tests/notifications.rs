use super::*;

use crate::notifications::NotificationEvent;
use rxqlite_common::{Message, MessageResponse};
use sqlx::types::chrono::{/*DateTime,*/ Utc};
use sqlx_sqlite_cipher::notifications::{Action, Notification};

fn do_notifications(test_name: &str, tls_config: Option<TestTlsConfig>) {
    let rt = Runtime::new().unwrap();

    let _ = rt.block_on(async {
        //const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
        let mut tm = TestManager::new(test_name, 3, tls_config);
        //tm.keep_temp_directories=true;
        tm.wait_for_cluster_established(1, 60).await.unwrap();
        let notifications_addr = tm.instances.get(&1).unwrap().notifications_addr.clone();
        let client = tm.clients.get_mut(&1).unwrap();

        client
            .start_listening_for_notifications(&notifications_addr)
            .await
            .unwrap();

        let message = Message::Execute(
            "CREATE TABLE IF NOT EXISTS _test_user_ (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      birth_date DATETIME NOT NULL
      )"
            .into(),
            vec![],
        );
        let response = client.sql(&message).await.unwrap();

        let message = response.data.unwrap();
        match message {
            MessageResponse::Rows(rows) => assert!(rows.len() == 0),
            MessageResponse::Error(err) => panic!("{}", err),
        }
        let name = "Ha";
        let birth_date = Utc::now();
        let message = Message::Execute(
            "INSERT INTO _test_user_ (name,birth_date) VALUES (?,?)".into(),
            vec![name.into(), birth_date.into()],
        );
        let response = client.sql(&message).await.unwrap();
        let message = response.data.unwrap();
        match message {
            MessageResponse::Rows(rows) => assert!(rows.len() == 0),
            MessageResponse::Error(err) => panic!("{}", err),
        }

        //tm.wait_for_last_applied_log(response.log_id,60).await.unwrap();

        // now we check for notification, will do with this:

        let notification_stream = client.notification_stream.as_mut().unwrap();
        let message = notification_stream
            .read_timeout(tokio::time::Duration::from_secs(60))
            .await
            .unwrap();
        assert!(message.is_some());
        match message.unwrap() {
            NotificationEvent::Notification(Notification::Update {
                action,
                database: _,
                table,
                row_id: _,
            }) => {
                assert_eq!(action, Action::SQLITE_INSERT);
                assert_eq!(&table, "_test_user_");
            }
        }
    });
}

#[test]
fn notifications() {
    do_notifications("notifications", None);
}

#[test]
fn notifications_insecure_ssl() {
    do_notifications(
        "notifications_insecure_ssl",
        Some(TestTlsConfig::default().accept_invalid_certificates(true)),
    );
}
