use rand::RngCore;

pub fn random_token(byte_len: usize) -> String {
    let mut buf = vec![0u8; byte_len];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn session_id() -> String {
    random_token(32)
}

pub fn csrf_token() -> String {
    random_token(24)
}
