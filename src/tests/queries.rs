use super::*;
use rxqlite_common::{Message,MessageResponse};
use sqlx::types::chrono::{DateTime,Utc};
use sqlx::{Pool,Row};
use sqlx_sqlite_cipher::{Sqlite};

use ring::digest;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::fs::File;
use std::io::BufReader;

fn do_simple_query(test_name:&str,tls_config: Option<TestTlsConfig>) {
  let rt = Runtime::new().unwrap();
  let _= rt.block_on(async {
    const QUERY: &str ="SELECT name,birth_date from _test_user_ where name = ?";
    let /*mut */tm = TestManager::new(test_name,3,tls_config);
    //used to check manually the TestManager directory
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
    //in tests , all instances share the same keys, so sqlite encryption key are 
    //common to all instances.
    
    let key_param = if tm.tls_config.is_some() {
        let private_key =
        rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(&mut File::open(&tm.key_path).unwrap())).filter_map(|x|x.ok())
        .next().unwrap();
            
        let hashed_key = digest::digest(&digest::SHA256, private_key.secret_pkcs8_der());
        
        let hashed_key = URL_SAFE.encode(hashed_key.as_ref());
        format!("?key=\"{}\"",hashed_key)
    } else {
      String::new()
    };
    
    for (node_id,_client) in tm.clients.iter() {
      let instance = tm.tcm.instances.get(node_id).unwrap();
      let sqlite_db_path = PathBuf::from(&instance.data_path).join(format!("sqlite.db{}",key_param));
      
      
      //we need to determine the database encryption key used by the rxqlited instance 
      //using the same algorithm that the one used in the instance
    
      
      
      //println!("sqlite_db_path for node {}: {}",node_id,sqlite_db_path.display());
      let pool = Pool::<Sqlite>::connect(sqlite_db_path.to_str().unwrap()).await.unwrap();
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

#[test]
fn simple_query() {
  do_simple_query("simple_query",None);
}

#[test]
fn simple_query_insecure_ssl() {
  do_simple_query("simple_query_insecure_ssl",Some(TestTlsConfig::default().accept_invalid_certificates(true)));
}
