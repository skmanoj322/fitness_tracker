use crate::models::{ExerciseLog, NewExerciseLog};
use crate::router::auth_handler;
use crate::utils::auth_midleware;
use axum::Json;
use axum::extract::Extension;
use axum::extract::{Query, Request, State};
use axum::middleware::from_fn;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

pub fn user_workout_log() -> Router<PgPool> {
    Router::new()
        .route("/new", get(handler))
        .route("/exercise", get(get_data))
        .route("/add", post(add_logs))
        .layer(from_fn(auth_midleware))
        .route("/auth/telegram", post(auth_handler))
}

#[derive(Serialize)]
struct MyResponse {
    hello: String,
}
#[derive(Deserialize)]
struct Params {
    id: Option<i32>,
}
async fn handler() -> impl IntoResponse {
    let response = MyResponse {
        hello: "world".to_string(),
    };
    Json(response)
}

async fn get_data(
    State(state): State<PgPool>,
    Query(params): Query<Params>,
    Extension(user_id): Extension<String>,
) -> Response {
    let telegram_id: i64 = serde_json::from_str::<serde_json::Value>(&user_id)
        .unwrap()
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap();
    let user = sqlx::query!("SELECT id FROM users WHERE telegram_id = $1", telegram_id)
        .fetch_one(&state)
        .await
        .unwrap();
    let user_id = user.id;

    if params.id.is_some() {
        let logs = sqlx::query_as!(
            ExerciseLog,
            "Select * from exercise_tracker where user_id=$1 and id=$2 ",
            user_id,
            params.id
        )
        .fetch_one(&state)
        .await
        .unwrap();

        return Json(logs).into_response();
    }
    let logs = sqlx::query_as!(
        ExerciseLog,
        "SELECT * FROM exercise_tracker WHERE
     user_id = $1",
        user_id
    )
    .fetch_all(&state)
    .await
    .unwrap();

    Json(logs).into_response()
}

async fn add_logs(
    State(state): State<PgPool>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<NewExerciseLog>,
) -> Response {
    let telegram_id: i64 = serde_json::from_str::<serde_json::Value>(&user_id)
        .unwrap()
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap();
    let user = sqlx::query!("SELECT id FROM users WHERE telegram_id = $1", telegram_id)
        .fetch_one(&state)
        .await
        .unwrap();

    let log = sqlx::query_as!(
        ExerciseLog,
        "INSERT INTO exercise_tracker (user_id, name, weight_kg, \"set\", rep) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        user.id as i32,
        payload.name,
        payload.weight_kg,
        payload.set,
        payload.rep,
    )
    .fetch_one(&state)
    .await
    .unwrap();

    return Json(json!({"status":201,"data":log})).into_response();
}
