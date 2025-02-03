use crate::model::repository::{Repository, approver_repository::ApproverRepository};
use bcrypt::verify;
use chrono::Local;

// Functions
pub async fn validate_code(code: String, destination: &String, repository: &ApproverRepository) -> Option<String> {
    let approvers = repository.find_all().await;
    let now = Local::now();

    for ap in approvers {
        if !ap.approved_types.contains(destination) { continue; }

        let validity_date = ap.validity.clone();
        if let Some(date) = validity_date {
            if now > date {
                continue;
            }   
        }

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
