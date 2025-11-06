use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(plain: &str) -> String {
    hash(plain, DEFAULT_COST).expect("hash failed")
}

pub fn verify_password(plain: &str, hashed: &str) -> bool {
    verify(plain, hashed).unwrap_or(false)
}
