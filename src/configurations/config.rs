
// Structs
struct ServerConfig {
    address: String,     
    port: usize,                   
    workers: usize,                 
    log_level: String,             
    keep_alive: usize,         
    secret_key: String 
}

struct DatabaseConfig {
    url: String,
    username: String,
    password: String
}

struct UnifiConfig {
    url: String,
    username: String,
    password: String,
}

struct ClientGroup {
    name: String,
    time_conneciton: usize,
    permissions: Vec<String>,
    restrictions: Vec<String>,
    upload_limit: usize,
    download_limit: usize,
}

struct ClientsConfig {
    groups: Vec<ClientGroup>
}

struct ApproversConfig {
    code_size: usize,
    validity_days_code: usize,
}

struct AdminsConfig {
    token_expirantion: usize,
} 

pub struct ConfigApplication {
    server: ServerConfig,
    unifi: UnifiConfig,
    database: DatabaseConfig,
    clients: ClientsConfig,
    approvers: ApproversConfig,
    admins: AdminsConfig
}

