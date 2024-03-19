use std::sync::Arc;

use warp::reply;

use crate::app::App;
use crate::Node;
use crate::TypeConfig;
use rxqlite_common::Message;
use openraft::LogId;
use openraft::LeaderId;

pub async fn sql(
    message : Message,
    app: Arc<App>,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    
    let sql = message.sql();
  
    let is_write = rxqlite_sqlx_common::is_query_write(sql);
    
    if is_write {
      
      let res: Result<openraft::raft::ClientWriteResponse<TypeConfig>,_>= app.raft.client_write(message).await;
      Ok(reply::json(&res))
    } else {
      let sqlite_and_path = app.sqlite_and_path.read().await;
      let response_message=rxqlite_sqlx_common::do_sql(&sqlite_and_path,message).await;
      let client_write_response = openraft::raft::ClientWriteResponse::<TypeConfig> {
        log_id : LogId {
          leader_id: LeaderId {
            term: u64::MAX,
            node_id: u64::MAX,
          },
          index: u64::MAX,
        },
        data: Some(response_message),
        membership: None,
      };
      let res = Result::<openraft::raft::ClientWriteResponse<TypeConfig>,
        openraft::error::RaftError<u64, openraft::error::ClientWriteError<u64, Node>>
        >::Ok(client_write_response);
      Ok(reply::json(&res))
    }
}


