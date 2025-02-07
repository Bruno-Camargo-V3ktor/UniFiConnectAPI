use rocket_db_pools::{Database, mongodb::Client};

// Structs
#[derive(Database)]
#[database("mongodb")]
pub struct MongoDb(Client);

// Módulo para lidar com conversões de ObjectId <-> String
pub mod serde_object_id {
    use bson::oid::ObjectId;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(id: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let object_id = ObjectId::parse_str(id).map_err(serde::ser::Error::custom)?;
        serializer.serialize_some(&object_id)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object_id: Result<ObjectId, <D as Deserializer<'_>>::Error> =
            Deserialize::deserialize(deserializer);

        match object_id {
            Ok(id) => Ok(id.to_string()),
            Err(_) => Ok(String::new()),
        }
    }
}
