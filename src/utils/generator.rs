use rand::prelude::*;

// Functions
pub fn generator_code(size: usize, just_numbers: bool) -> String {
    if just_numbers {
        let charset = b"0123456789";
        let mut rng = rand::rng();
        let mut code = String::new();

        for _ in 0..size {
            let i = rng.random_range(0..charset.len());
            let ch = charset[i] as char;
            code.push(ch);
        }

        return code;
    }

    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789";
    let mut rng = rand::rng();
    let mut code = String::new();

    for _ in 0..size {
        let i = rng.random_range(0..charset.len());
        let ch = charset[i] as char;
        code.push(ch);
    }

    code
}
