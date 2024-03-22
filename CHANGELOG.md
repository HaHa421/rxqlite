# Changelog

## [0.1.2] - 2024-03-21

### Added
Insecure Tls Support: 
  rxqlite support tls both for API and Intra Nodes 
  communication accepting any certificate 
  and therefore self signed certificates.
  
### Modified
Removed the init cluster api:
  Nodes are initiated through command line arguments.
  