use regex::Regex;

// Structs
pub struct Validator;

// Impls
impl Validator {
    pub fn validate_phone(phone: &String) -> bool {
        if phone.len() != 11 {
            return false;
        }
        phone.chars().all(|c| c.is_numeric())
    }

    pub fn validate_cpf(cpf: &Option<String>) -> bool {
        let cpf = if cpf.is_some() {
            cpf.clone().unwrap()
        } else {
            return true;
        };

        if cpf.len() != 11 {
            return false;
        }

        let mut digits = (&cpf[0..9]).chars().collect::<Vec<char>>();

        // First Digit
        let base: usize = 10;
        let mut count = 0;

        for i in 0..digits.len() {
            let d = digits[i].to_digit(10).unwrap() * (base - i) as u32;
            count += d;
        }
        let res = count % 11;

        if res < 2 {
            digits.push('0');
        } else {
            let r = (11 - res) as u8;
            if r >= 10 {
                digits.push('0');
            } else {
                digits.push((r + b'0') as char);
            }
        }

        // Second Digit
        let base: usize = 11;
        let mut count = 0;

        for i in 0..digits.len() {
            let d = digits[i].to_digit(10).unwrap() * (base - i) as u32;
            count += d;
        }
        let res = count % 11;

        if res < 2 {
            digits.push('0');
        } else {
            let r = (11 - res) as u8;
            if r >= 10 {
                digits.push('0');
            } else {
                digits.push((r + b'0') as char);
            }
        }

        digits.iter().collect::<String>() == cpf.clone()
    }

    pub fn validate_email(email: &String) -> bool {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email.as_str())
    }
}
