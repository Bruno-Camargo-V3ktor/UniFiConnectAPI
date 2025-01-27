// Structs
pub struct Validator;

// Impls
impl Validator {
    pub fn validate_phone(phone: String) -> bool {
        if phone.len() != 11 {
            return false;
        }
        phone.chars().all(|c| c.is_numeric())
    }
}
