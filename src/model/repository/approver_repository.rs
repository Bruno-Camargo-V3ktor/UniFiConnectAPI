use super::Repository;
use crate::{db::mongo_db::MongoDb, model::entity::approver::Approver};
use bson::{Document, doc, oid::ObjectId, to_document};
use rocket::request::{FromRequest, Outcome, Request};
use rocket_db_pools::{Connection, mongodb::Database};

// Structs
pub struct ApproverRepository {
    database: Database,
    name: String,
}

// Impls
impl Repository for ApproverRepository {
    type Id = String;
    type Entity = Approver;
    type Options = Document;

    async fn find(&self, query: Self::Options) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);
        let res = collection.find(query, None).await;

        match res {
            Ok(mut c) => {
                let mut entitys = vec![];
                while let Ok(_) = c.advance().await {
                    let e = c.deserialize_current().unwrap();
                    entitys.push(e);
                }

                entitys
            }

            Err(_) => {
                vec![]
            }
        }
    }

    async fn find_one(&self, query: Self::Options) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);
        let res = collection.find_one(query, None).await;

        match res {
            Ok(op) => op,
            Err(_) => None,
        }
    }

    async fn find_by_id(&self, id: Self::Id) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);

        let doc = doc! {
            "_id": ObjectId::parse_str(&id).unwrap(),
        };

        let res = collection.find_one(doc, None).await;
        if let Ok(e) = res { e } else { None }
    }

    async fn find_all(&self) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);

        let res = collection.find(doc! {}, None).await;

        match res {
            Ok(mut op) => {
                let mut entitys = vec![];
                while let Ok(_) = op.advance().await {
                    let e = op.deserialize_current().unwrap();
                    entitys.push(e);
                }

                entitys
            }

            Err(_) => vec![],
        }
    }

    async fn save(&self, mut entity: Self::Entity) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);

        let new_id = ObjectId::new().to_string();
        entity.id = new_id;

        let res = collection.insert_one(&entity, None).await;

        if let Ok(_) = res { Some(entity) } else { None }
    }

    async fn save_all(&self, mut entitys: Vec<Self::Entity>) -> Vec<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);

        for e in entitys.iter_mut() {
            let new_id = ObjectId::new().to_string();
            e.id = new_id;
        }

        let _ = collection.insert_many(&entitys, None).await;
        entitys
    }

    async fn update(&self, entity: Self::Entity) -> Option<Self::Entity> {
        let collection = self.database.collection::<Self::Entity>(&self.name);

        let res = collection
            .update_one(
                doc! { "_id" : ObjectId::parse_str( &entity.id ).unwrap() },
                to_document(&entity).unwrap(),
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
        let collection = self.database.collection::<Self::Entity>(&self.name);

        let res = collection.update_one(query, modify, None).await;
        match res {
            Ok(r) => r.modified_count as usize,
            Err(_) => 0,
        }
    }

    async fn delete(&self, entity: Self::Entity) -> bool {
        let collection = self.database.collection::<Self::Entity>(&self.name);
        let res = collection
            .delete_one(
                doc! { "_id" : ObjectId::parse_str(&entity.id).unwrap() },
                None,
            )
            .await;

        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn delete_by_id(&self, id: Self::Id) -> bool {
        let collection = self.database.collection::<Self::Entity>(&self.name);
        let res = collection
            .delete_one(doc! { "_id" : ObjectId::parse_str(&id).unwrap() }, None)
            .await;

        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn delete_all(&self, query: Self::Options) -> usize {
        let collection = self.database.collection::<Self::Entity>(&self.name);
        let res = collection.delete_one(query, None).await;

        match res {
            Ok(r) => r.deleted_count as usize,
            Err(_) => 0,
        }
    }
}

// Guards
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApproverRepository {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let db = request.guard::<Connection<MongoDb>>().await.unwrap();
        let repository = ApproverRepository {
            database: db.default_database().unwrap(),
            name: "Approvers".to_string(),
        };

        Outcome::Success(repository)
    }
}
