use num_bigint::BigInt;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use thiserror::Error;
use uuid::Uuid;

const AUTH_BASE_URL: &str = "https://sessionserver.mojang.com/session/minecraft/hasJoined";

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("invalid session")]
    InvalidSession,

    #[error("request error")]
    Request(#[from] Box<ureq::Error>),

    #[error("deserialization error")]
    Deserialization(#[from] std::io::Error),
}

#[derive(Debug, Deserialize)]
pub struct AuthenticationResponse {
    pub id: Uuid,
    pub properties: Vec<PlayerProperty>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

fn hash(server_id: &str, shared_secret: &[u8], encoded_public_key: &[u8]) -> String {
    let mut sha1 = Sha1::new();
    sha1.update(server_id.as_bytes());
    sha1.update(shared_secret);
    sha1.update(encoded_public_key);
    format!("{:x}", BigInt::from_signed_bytes_be(&sha1.finalize()))
}

pub fn authenticate(
    server_id: &str,
    shared_secret: &[u8],
    encoded_public_key: &[u8],
    username: &str,
) -> Result<AuthenticationResponse, AuthenticationError> {
    let hash = hash(server_id, shared_secret, encoded_public_key);

    let response = ureq::get(&format!(
        "{}?username={}&serverId={}",
        AUTH_BASE_URL, username, hash
    ))
    .call()
    .map_err(Box::new)?;

    if response.status() != 200 {
        return Err(AuthenticationError::InvalidSession);
    }

    Ok(response.into_json()?)
}
