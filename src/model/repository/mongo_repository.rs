use std::marker::PhantomData;

use super::Repository;
use crate::{db::mongo_db::MongoDb, model::entity::Entity};
use bson::{Document, doc, oid::ObjectId, to_document};
use rocket::{
    futures::TryStreamExt,
    request::{FromRequest, Outcome, Request},
    serde::{Serialize, DeserializeOwned}
};
use rocket_db_pools::{Connection, mongodb::Database};

// Structs
pub struct MongoRepository<E> {
    pub database: Database,
    pub _phantom: PhantomData<E>
}

// Impls
impl<E: Entity<String> + Serialize  + DeserializeOwned + Unpin + Send + Sync> MongoRepository<E> {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            _phantom: PhantomData
        }
    }
}

impl<E: Entity<String> + Serialize  + DeserializeOwned + Unpin + Send + Sync> Repository for MongoRepository<E> {
    type Id = String;
    type Entity = E;
    type Options = Document;

    async fn find(&self, query: Self::Options) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());
        let res = collection.find(query, None).await;

        match res {
            Ok(mut cursor) => {
                let mut entitys = vec![];
                while let Ok(Some(e)) = cursor.try_next().await {
                    entitys.push(e);
                }

                entitys
            }

            Err(_) => vec![],
        }
    }

    async fn find_one(&self, query: Self::Options) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());
        let res = collection.find_one(query, None).await;
        
        res.unwrap_or_default()
    }

    async fn find_by_id(&self, id: Self::Id) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        let doc = doc! {
            "_id": ObjectId::parse_str(&id).unwrap(),
        };

        let res = collection.find_one(doc, None).await;
        res.unwrap_or_default()
    }

    async fn find_all(&self) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        let res = collection.find(doc! {}, None).await;

        match res {
            Ok(mut cursor) => {
                let mut entitys = vec![];
                while let Ok(Some(e)) = cursor.try_next().await {
                    entitys.push(e);
                }

                entitys
            }

            Err(_) => vec![],
        }
    }

    async fn save(&self, mut entity: Self::Entity) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        let new_id = ObjectId::new().to_string();
        entity.set_id(new_id);

        let res = collection.insert_one(&entity, None).await;

        if res.is_ok() {
            return Some(entity)
        }

        None
    }

    async fn save_all(&self, mut entitys: Vec<Self::Entity>) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        for e in entitys.iter_mut() {
            let new_id = ObjectId::new().to_string();
            e.set_id(new_id);
        }

        let _ = collection.insert_many(&entitys, None).await;
        entitys
    }

    async fn update(&self, entity: Self::Entity) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        let res = collection
            .update_one(
                doc! { "_id" : ObjectId::parse_str( entity.get_id() ).unwrap() },
                doc! { "$set": to_document(&entity).unwrap() },
                None,
            )
            .await;

        if let Ok(r) = res {
            if r.modified_count != 0 {
                return Some(entity);
            }
        }

        None
    }

    async fn update_all(&self, query: Self::Options, modify: Self::Options) -> usize {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());

        let res = collection.update_many(query, modify, None).await;
        match res {
            Ok(r) => r.modified_count as usize,
            Err(_) => 0,
        }
    }

    async fn delete(&self, entity: Self::Entity) -> bool {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());
        let res = collection
            .delete_one(
                doc! { "_id" : ObjectId::parse_str( entity.get_id() ).unwrap() },
                None,
            )
            .await;
        
        res.is_ok()
    }

    async fn delete_by_id(&self, id: Self::Id) -> bool {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());
        let res = collection
            .delete_one(doc! { "_id" : ObjectId::parse_str(&id).unwrap() }, None)
            .await;


        res.is_ok()
    }

    async fn delete_all(&self, query: Self::Options) -> usize {
        let collection = self.database.collection::<Self::Entity>(&Self::Entity::get_name());
        let res = collection.delete_one(query, None).await;

        match res {
            Ok(r) => r.deleted_count as usize,
            Err(_) => 0,
        }
    }
}

// Guards
#[rocket::async_trait]
impl<'r, E: Entity<String> + Serialize  + DeserializeOwned + Unpin + Send + Sync> FromRequest<'r> for MongoRepository<E> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = request.guard::<Connection<MongoDb>>().await.unwrap();

        let repository =  MongoRepository {
            database: db.default_database().unwrap(),
            _phantom: PhantomData
        };
        Outcome::Success(repository)
    }
}
