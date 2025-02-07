pub mod admin;
pub mod approver;
pub mod client;
pub mod user;

// Traits
pub trait Entity<I> {
    fn get_id(&self) -> I;
    fn set_id(&mut self, new_id: I);
    fn get_name() -> String;
}
