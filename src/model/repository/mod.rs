// Mods
pub mod admin_repositoy;
pub mod approver_repository;
pub mod guest_repository;

// Traits
#[allow(unused)]
pub trait Repository {
    type Id;
    type Entity;
    type Options;

    async fn find_by_id(&self, id: Self::Id) -> Option<Self::Entity> {
        todo!()
    }
    async fn find_one(&self, query: Self::Options) -> Option<Self::Entity> {
        todo!()
    }
    async fn find(&self, query: Self::Options) -> Vec<Self::Entity> {
        todo!()
    }
    async fn find_all(&self) -> Vec<Self::Entity> {
        todo!()
    }

    async fn save(&self, entity: Self::Entity) -> Option<Self::Entity> {
        todo!()
    }
    async fn save_all(&self, entitys: Vec<Self::Entity>) -> Vec<Self::Entity> {
        todo!()
    }

    async fn update(&self, entity: Self::Entity) -> Option<Self::Entity> {
        todo!()
    }
    async fn update_all(&self, query: Self::Options, modify: Self::Options) -> usize {
        todo!()
    }

    async fn delete(&self, entity: Self::Entity) -> bool {
        todo!()
    }
    async fn delete_by_id(&self, id: Self::Id) -> bool {
        todo!()
    }
    async fn delete_all(&self, query: Self::Options) -> usize {
        todo!()
    }
}
