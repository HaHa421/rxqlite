#![deny(warnings)]

use clap::Parser;
use rxqlite::{init_example_raft_node, start_example_raft_node};
//use tracing_subscriber::EnvFilter;
use rxqlite_common::RSQliteNodeTlsConfig;
//use openraft::NodeId;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Opt {
    #[clap(long)]
    pub id: u64,

    #[clap(long)]
    pub http_addr: Option<String>,

    #[clap(long)]
    pub rpc_addr: Option<String>,

    #[clap(long,action = clap::ArgAction::SetTrue)]
    leader: Option<bool>,

    #[clap(long, action = clap::ArgAction::Append)]
    member: Vec<String>, // id;http_addr;rpc_addr

    #[clap(long)]
    key_path: Option<String>,

    #[clap(long)]
    cert_path: Option<String>,

    #[clap(long,action = clap::ArgAction::SetTrue)]
    accept_invalid_certificates: Option<bool>,

    #[clap(long,action = clap::ArgAction::SetTrue)]
    no_database_encryption: Option<bool>,

    #[clap(long)]
    notifications_addr: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup the logger
    /*
    tracing_subscriber::fmt()
        //.with_max_level(tracing::Level::TRACE)
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    */

    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(true)
        //.with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse the parameters passed by arguments.
    let options = Opt::parse();

    let base_path = std::path::PathBuf::from(format!("data-{}", options.id));

    let tls_config = if options.key_path.is_some() && options.cert_path.is_some() {
        Some(RSQliteNodeTlsConfig {
            key_path: options.key_path.unwrap(),
            cert_path: options.cert_path.unwrap(),
            accept_invalid_certificates: options.accept_invalid_certificates.unwrap_or(false),
        })
    } else {
        None
    };
    //options.accept_invalid_certificates,

    if base_path.is_dir() {
        start_example_raft_node(
            options.id,
            base_path,
            options.http_addr,
            options.rpc_addr,
            options.notifications_addr,
            tls_config,
            options.no_database_encryption.unwrap_or(false),
        )
        .await?;
        Ok(())
    } else {
        let leader = options.leader.unwrap_or(false);
        if !leader && options.member.len() > 0 {
            return Err(anyhow::anyhow!(
                "members can be specified on the leader node only"
            ));
        }
        let mut members = vec![];
        for member in options.member.into_iter() {
            let mut elements = member.split(";");
            let node_id = if let Some(node_id_str) = elements.next() {
                match node_id_str.parse::<u64>() {
                    Ok(node_id) => node_id,
                    Err(r) => {
                        return Err(anyhow::anyhow!(format!(
                            "couldn't parse member id from: {}({})",
                            node_id_str, r
                        )));
                    }
                }
            } else {
                return Err(anyhow::anyhow!(
                    "member must be provided in the form 'node_id;http_addr;rpc_addr'"
                ));
            };
            let http_addr = if let Some(http_addr_str) = elements.next() {
                http_addr_str.to_string()
            } else {
                return Err(anyhow::anyhow!(
                    "member must be provided in the form 'node_id;http_addr;rpc_addr'"
                ));
            };
            let rpc_addr = if let Some(http_addr_str) = elements.next() {
                http_addr_str.to_string()
            } else {
                return Err(anyhow::anyhow!(
                    "member must be provided in the form 'node_id;http_addr;rpc_addr'"
                ));
            };
            if elements.next().is_some() {
                return Err(anyhow::anyhow!(
                    "member must be provided in the form 'node_id;http_addr;rpc_addr'"
                ));
            }
            members.push((node_id, http_addr, rpc_addr));
        }
        init_example_raft_node(
            options.id,
            base_path,
            leader,
            options.http_addr,
            options.rpc_addr,
            options.notifications_addr,
            members,
            tls_config,
            options.no_database_encryption.unwrap_or(false),
        )
        .await
    }
}
