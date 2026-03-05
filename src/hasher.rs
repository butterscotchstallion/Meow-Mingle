use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

#[allow(dead_code)]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "c0rrectP@ssw0rd";
        let hash = hash_password(password).expect("Hashing failed");

        assert!(verify_password(password, &hash).expect("Verification failed"));
    }

    #[test]
    fn test_wrong_password_fails_verification() {
        let hash = hash_password("c0rrectP@ssw0rd").expect("Hashing failed");

        assert!(!verify_password("wrongpassword", &hash).expect("Verification failed"));
    }

    #[test]
    fn test_hashes_are_unique() {
        let password = "s@meP@ssw0rd";
        let hash1 = hash_password(password).expect("Hashing failed");
        let hash2 = hash_password(password).expect("Hashing failed");

        assert_ne!(
            hash1, hash2,
            "Two hashes of the same password should differ due to unique salts"
        );
    }
}
