// checks the jwt from header if it exists it will verify it else it will throw the error unauthrized

use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::utils::decode_token;

pub async fn auth_midleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let authrization = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok());

    let token = match authrization {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token = token
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match decode_token(token.to_string()) {
        Ok(claims) => {
            req.extensions_mut().insert(claims.user_id);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
