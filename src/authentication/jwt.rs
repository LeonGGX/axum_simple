//! src/authentication/jwt.rs

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub user_role: String,
    pub expires_in: Option<i64>,
}

impl TokenDetails {
    pub fn new(
        token: Option<String>,
        token_uuid: Uuid,
        user_id: Uuid,
        user_role: String,
        expires_in: Option<i64>,
    ) -> Self {
        Self {
            token,
            token_uuid,
            user_id,
            user_role,
            expires_in,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub role: String,
    pub token_uuid: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}
///
/// generate_jwt_token function.    
/// It takes in:    
/// a user ID,    
/// a time-to-live (TTL) value for the token,    
/// and a private key as input parameters.
///    
/// In this function, we first decode the base64-encoded private key back to a UTF8 string.
/// After decoding the base64-encoded private key, we create instances of the TokenDetails and TokenClaims structs
/// that respectively hold the metadata and claims of the JWT.
/// These structs are then passed to the jsonwebtoken::encode() function,
/// which generates the JWT using the RS256 algorithm.
///
/// Finally, we add the generated token to the TokenDetails struct and return it from the function.
///
#[allow(dead_code)]
pub fn generate_jwt_token(
    user_id: uuid::Uuid,
    user_role: String,
    ttl: i64,
    private_key: String,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_private_key = general_purpose::STANDARD.decode(private_key).unwrap();
    let decoded_private_key = String::from_utf8(bytes_private_key).unwrap();

    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        user_id,
        user_role,
        token_uuid: Uuid::new_v4(),
        expires_in: Some((now + chrono::Duration::minutes(ttl)).timestamp()),
        token: None,
    };

    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        role: token_details.user_role.to_string(),
        token_uuid: token_details.token_uuid.to_string(),
        exp: token_details.expires_in.unwrap(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(decoded_private_key.as_bytes())?,
    )?;
    token_details.token = Some(token);
    Ok(token_details)
}
///
/// This function takes a public key and a token as input parameters.    
/// The first step is to decode the base64-encoded public key back to a UTF8 string.
///
/// After decoding the public key, we create a new instance of the jsonwebtoken::Validation struct,
/// where we specify the RS256 algorithm used in signing the JWT.
///
/// Then, we call the jsonwebtoken::decode() function,
/// which decodes the JWT using the provided public key and validation parameters.
/// The result is stored in a decoded variable, which contains the claims of the JWT.
///
/// As the ‘sub‘ and ‘token_uuid‘ fields of the TokenClaims struct are stored as strings,
/// we need to convert them into UUID types.
/// To do this, we use the Uuid::parse_str() function
/// and assign the parsed values to the ‘user_id‘ and ‘token_uuid‘ variables, respectively.
///
/// Finally, we create a new instance of the TokenDetails struct
/// with the parsed user_id and token_uuid values,
/// set the expires_in and token fields to ‘None‘, and return it from the function.
///
pub fn verify_jwt_token(
    public_key: String,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_public_key = general_purpose::STANDARD.decode(public_key).unwrap();
    let decoded_public_key = String::from_utf8(bytes_public_key).unwrap();

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
        &validation,
    )?;

    let user_id = Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();
    let user_role = decoded.claims.role;
    let token_uuid = Uuid::parse_str(decoded.claims.token_uuid.as_str()).unwrap();

    Ok(TokenDetails {
        token: None,
        token_uuid,
        user_id,
        user_role,
        expires_in: None,
    })
}
