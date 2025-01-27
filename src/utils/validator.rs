// Structs
pub struct Validator;

// Impls
impl Validator {
    pub fn validate_phone(phone: String) -> bool {
        if phone.len() == 11 { true } else { false }
    }
}
