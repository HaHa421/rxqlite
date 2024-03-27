# Changelog

## [0.1.8] - 2024-03-27
Introduced the notifcation system
rxqlited now listens on three ports:
host-addr is the address the node listens to api calls
rpc-addr is the address the node listens to raft(internal) calls
notifications-addr is the address the node listens to notifications messages

Notifications are a subset of sqlite hooks and lets the client get notfied on database modifications.

## [0.1.7] - 2024-03-25
optimized and upgraded rocksdb value encryption to AES-GCM-SIV

## [0.1.6] - 2024-03-25
Added initial databases encryption (rocksdb values and sqlite)

## [0.1.5] - 2024-03-23
Added initial test infrastructure

## [0.1.4] - 2024-03-22
API: 
Added sql_consistent: 
with sql_consistent <b>read</b> queries are executed on the leader to ensure the 
query is executed with the current state of the cluster,
while sql <b>read</b> queries are executed on whatever node the client connects to,
without redirecting to the leader, data might not be reflecting the leader state (replication is not instantaneous) , but the query is faster.
NB: <b>read</b> only queries are not written in the log.

<b>write</b> queries are always executed on the leader.
Startup:
Node startup has been simplified: when the cluster has been already initialized, 
on next startups, a node needs only its node id to launch:
This is shown in ha-init-cluster.sh or ha-init-cluster-insecure.sh, and ha-start-cluster.sh (when you have rebooted the machine or `killed` the cluster using ha-kill-cluster.sh)

## [0.1.3] - 2024-03-21


## [0.1.2] - 2024-03-21

### Added
Insecure Tls Support: 
  rxqlite support tls both for API and Intra Nodes 
  communication accepting any certificate 
  and therefore self signed certificates.
  
### Modified
Removed the init cluster api:
  Nodes are initiated through command line arguments.
  