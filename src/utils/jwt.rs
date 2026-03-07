use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::utils::{create_secret_key, parse_string, verify_signature};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    exp: usize,
    iat: usize,
}

pub fn encode_claims(user_id: String) -> Result<String, Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp() as usize;

    let secret = std::env::var("JWT_SECRET").unwrap();

    let claims = Claims {
        user_id,
        iat: now,
        exp: now + 60 * 60 * 24,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    );

    token
}

pub fn decode_token(token: String) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").unwrap();
    let token_message = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )?;

    Ok(token_message.claims)
}

pub fn deserialize_user<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

async fn jwt_token(jwt_token: &str) -> bool {
    let token_result = decode_token(jwt_token.to_string());

    if token_result.is_ok() {
        return true;
    }

    false
}

pub async fn isvalid_init_data(val: &str) -> bool {
    if val.starts_with("tma ") {
        if let Some(auth_token) = val.strip_prefix("tma ") {
            let bot_data = std::env::var("BOT_TOKEN").unwrap();

            let (init_data, hash) = parse_string(auth_token);

            let secret = create_secret_key(&bot_data);

            let is_valid_token = verify_signature(&secret, &init_data, &hash);
            if is_valid_token {
                // check the user id exist in db
                // if it does generate the token
                // if it doesnt inset it into db and then generate the token

                return true;
            }
        }
    }

    return false;
}
