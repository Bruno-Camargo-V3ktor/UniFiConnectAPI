use crate::model::repository::{Repository, approver_repository::ApproverRepository};
use bcrypt::verify;

// Functions
pub async fn validate_code(code: String, repository: &ApproverRepository) -> Option<String> {
    let approvers = repository.find_all().await;

    for ap in approvers {
        let res = verify(&code, ap.secrete_code.as_str());
        match res {
            Ok(b) => {
                if b {
                    return Some(ap.username);
                } else {
                    continue;
                }
            }

            Err(_) => {
                continue;
            }
        }
    }

    None
}
