use super::*;
use rxqlite_common::{Message,MessageResponse};
use sqlx::types::chrono::{DateTime,Utc};
use sqlx::{SqlitePool,Row};


#[test]
fn simple_query() {
  let rt = Runtime::new().unwrap();
  let _= rt.block_on(async {
    const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
    let /*mut*/ tm = TestManager::new("simple_query",3,None);
    //tm.keep_temp_directories=true;
    tm.wait_for_cluster_established(1,60).await.unwrap();
    let client = tm.clients.get(&1).unwrap();
    
    let message=Message::Execute(
      "CREATE TABLE IF NOT EXISTS _test_user_ (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL UNIQUE,
      birth_date DATETIME NOT NULL
      )".into(),
      vec![]
      );
    let response = client.sql(&message).await.unwrap();
    
    let message = response.data.unwrap();
    match message {
      MessageResponse::Rows(rows)=>assert!(rows.len()==0),
      MessageResponse::Error(err)=>panic!("{}",err),
    }
    let name = "Ha";
    let birth_date = Utc::now();
    let message=Message::Execute(
      "INSERT INTO _test_user_ (name,birth_date) VALUES (?,?)".into(),
      vec![name.into(),birth_date.into()]
    );
    let response = client.sql(&message).await.unwrap();
    let message = response.data.unwrap();
    match message {
      MessageResponse::Rows(rows)=>assert!(rows.len()==0),
      MessageResponse::Error(err)=>panic!("{}",err),
    }
    
    tm.wait_for_last_applied_log(response.log_id,60).await.unwrap();
    
    //now check that query has been applied to each instance local sqlite database
    
    for (node_id,_client) in tm.clients.iter() {
      let instance = tm.tcm.instances.get(node_id).unwrap();
      let sqlite_db_path = PathBuf::from(&instance.data_path).join("sqlite.db");
      //println!("sqlite_db_path for node {}: {}",node_id,sqlite_db_path.display());
      let pool = SqlitePool::connect(sqlite_db_path.to_str().unwrap()).await.unwrap();
      let rows=sqlx::query(QUERY)
        .bind("Ha")
        .fetch_all(&pool)
        .await
        .unwrap();
      assert!(rows.len()==1);
      let row = &rows[0];
      let fetched_name:String = row.get(0);
      assert_eq!(&fetched_name,name);
      let fetched_birth_date:DateTime<Utc> = row.get(1);
      assert_eq!(fetched_birth_date,birth_date);
    }
    
    //now check query through rxqlited
    
    let message=Message::Fetch(
      QUERY.into(),
      vec![name.into()]
    );
    let response = client.sql(&message).await.unwrap();
    let message = response.data.unwrap();
    match message {
      MessageResponse::Rows(rows)=>{
        assert_eq!(rows.len(),1);
        let row = &rows[0];
        let fetched_name:String = row.get(0);
        assert_eq!(&fetched_name,name);
        let fetched_birth_date:DateTime<Utc> = row.get(1);
        assert_eq!(fetched_birth_date,birth_date);
      }
      MessageResponse::Error(err)=>panic!("{}",err),
    }
  });
}
