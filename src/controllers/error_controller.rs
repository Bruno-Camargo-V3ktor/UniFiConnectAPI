use crate::utils::error::{Error, NotFound};
use rocket::{Catcher, Request, catch, catchers};

#[catch(404)]
fn api_not_found(req: &Request) -> NotFound {
    Error::new_not_found(
        format!(
            "No controller found for URL: `{}` ",
            req.uri().path().as_str()
        )
        .as_str(),
    )
}

// Functions
pub fn handles() -> Vec<Catcher> {
    catchers![api_not_found]
}
