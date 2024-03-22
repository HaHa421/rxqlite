use std::sync::Arc;

use warp::reply;

use crate::app::App;
use crate::Node;
use crate::TypeConfig;
use rxqlite_common::Message;
use openraft::LogId;
use openraft::LeaderId;

pub async fn sql_consistent_or_fast(
    message : Message,
    app: Arc<App>,
    consistent: bool,
) -> Result<impl warp::Reply, std::convert::Infallible> {
  let sql = message.sql();
  
    let is_write = rxqlite_sqlx_common::is_query_write(sql);
    
    if is_write {
      
      let res: Result<openraft::raft::ClientWriteResponse<TypeConfig>,_>= app.raft.client_write(message).await;
      Ok(reply::json(&res))
    } else {
      let do_it_locally = if consistent {
        if let Ok(_read_log_id) = app.raft.ensure_linearizable().await {
          true
        } else {
          false
        }
      } else {
        true
      };
      if do_it_locally {
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
      } else {
        let server_metrics = app.raft.server_metrics().borrow().clone();
        if let Some(leader_id) = server_metrics.current_leader {
          let membership_config=server_metrics.membership_config;
          let leader_node=membership_config.nodes().find_map(|(node_id,node)| if node_id == &leader_id {
            Some(node.clone())
          } else {
            None
          });
          let res = Result::<openraft::raft::ClientWriteResponse<TypeConfig>,
            openraft::error::RaftError<u64, openraft::error::ClientWriteError<u64, Node>>
            >::Err(
            openraft::error::RaftError::APIError(
              openraft::error::ClientWriteError::ForwardToLeader(
                openraft::error::ForwardToLeader {
                  leader_id: Some(leader_id),
                  leader_node: leader_node,
                }
              )
            )
          );
          Ok(reply::json(&res))
        } else {
          let res = Result::<openraft::raft::ClientWriteResponse<TypeConfig>,
            openraft::error::RaftError<u64, openraft::error::ClientWriteError<u64, Node>>
            >::Err(
            openraft::error::RaftError::APIError(
              openraft::error::ClientWriteError::ForwardToLeader(
                openraft::error::ForwardToLeader {
                  leader_id: None,
                  leader_node: None,
                }
              )
            )
          );
          Ok(reply::json(&res))
        }
      }
    }
}

pub async fn sql(
    message : Message,
    app: Arc<App>,
) -> Result<impl warp::Reply, std::convert::Infallible> {
  sql_consistent_or_fast(message,app,false).await
    
}

pub async fn sql_consistent(
    message : Message,
    app: Arc<App>,
) -> Result<impl warp::Reply, std::convert::Infallible> {
  sql_consistent_or_fast(message,app,true).await 
}

