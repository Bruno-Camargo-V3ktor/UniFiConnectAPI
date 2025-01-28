use crate::utils::error::{BadRequest, Error, NotFound};
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

#[catch(400)]
fn api_bad_request(_req: &Request) -> BadRequest {
    Error::new_bad_request("Invalid request body")
}

// Functions
pub fn handles() -> Vec<Catcher> {
    catchers![api_bad_request, api_not_found]
}
