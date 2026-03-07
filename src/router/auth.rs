use crate::{
    models::InitData,
    utils::{encode_claims, isvalid_init_data},
};
use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use serde_json::json;
use sqlx::{PgPool, query};

// if user exist the just responde with jwt
// else save it in db  and responde with jwt

pub async fn auth_handler(State(state): State<PgPool>, req: Request) -> Response {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED);
    if let Ok(init_data) = auth_header {
        if isvalid_init_data(init_data).await {
            let init_parsed_data = parse_init_data(init_data).unwrap();

            let user_exist = query!(
                "select * from users where telegram_id=$1",
                init_parsed_data.user.id as i64
            )
            .fetch_optional(&state)
            .await
            .unwrap();

            if let Some(user) = user_exist {
                println!("User ,exist{:?}", user);
            } else {
                query!(
                        "INSERT INTO users (telegram_id, first_name, last_name, user_name) VALUES ($1, $2, $3, $4)",
                        init_parsed_data.user.id as i64,
                        init_parsed_data.user.first_name,
                        init_parsed_data.user.last_name,
                        init_parsed_data.user.user_name
                    )
                    .execute(&state)
                    .await
                    .unwrap();
            }

            let token =
                encode_claims(serde_json::to_string(&init_parsed_data.user).unwrap()).unwrap();

            return (StatusCode::OK, Json(json!({"token":token}))).into_response();
        }
    }

    // take init data check if exist in db and generate the jwt and pass it response body

    StatusCode::UNAUTHORIZED.into_response()
}

fn parse_init_data(raw: &str) -> Result<InitData, Box<dyn std::error::Error>> {
    let init_data: InitData = serde_urlencoded::from_str(raw)?;

    println!("{:?}", init_data);

    Ok(init_data)
}
