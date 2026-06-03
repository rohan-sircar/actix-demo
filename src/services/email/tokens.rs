pub fn generate_token() -> String {
    let bytes: [u8; 32] = rand::random();
    data_encoding::BASE64URL_NOPAD.encode(&bytes)
}

pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
