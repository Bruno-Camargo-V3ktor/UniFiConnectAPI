use rocket_db_pools::{Database, mongodb::Client};

#[derive(Database)]
#[database("mongodb")]
pub struct MongoDb(Client);
