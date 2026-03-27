use std::collections::HashMap;

use crate::models::{DeleteLog, EditExerciseLog, ExerciseLog, NewExerciseLog};
use crate::router::auth_handler;
use crate::utils::auth_midleware;
use axum::extract::Extension;
use axum::extract::{Query, Request, State};
use axum::middleware::from_fn;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Error, Json};
use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Pool, Postgres, query_as};

pub fn user_workout_log() -> Router<PgPool> {
    Router::new()
        .route("/new", get(handler))
        .route("/exercise", get(get_data))
        .route("/getSession", get(get_session_logs))
        .route("/add", post(add_logs))
        .route("/edit", post(edit_logs))
        .route("/delete", post(delete_logs))
        .route("/sendMessage", get(sendmessage))
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

async fn get_user_id(user_id: String, state: &Pool<Postgres>) -> i32 {
    let telegram_id: i64 = serde_json::from_str::<serde_json::Value>(&user_id)
        .unwrap()
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap();

    let user = sqlx::query!("SELECT id FROM users WHERE telegram_id = $1", telegram_id)
        .fetch_one(state)
        .await
        .unwrap();

    user.id
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

async fn edit_logs(
    State(state): State<PgPool>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<EditExerciseLog>,
) -> Response {
    let user_id = get_user_id(user_id, &state).await;

    let edit_log = sqlx::query_as!(
        ExerciseLog,
        "update exercise_tracker set name=$1, weight_kg=$2,\"set\"=$3,rep=$4 where id=$5 and user_id=$6 RETURNING *",
        payload.name,
        payload.weight_kg,
        payload.set,
        payload.rep,
        payload.id,
        user_id
    )
    .fetch_one(&state)
    .await
    .unwrap();

    return Json(json!({"stautus":201,"data":edit_log})).into_response();
}

async fn delete_logs(
    State(state): State<PgPool>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<DeleteLog>,
) -> Response {
    let user_id = get_user_id(user_id, &state).await;
    let delete_log = query_as!(
        ExerciseLog,
        "delete from exercise_tracker where user_id=$1 and id=$2 RETURNING *",
        user_id,
        payload.id,
    )
    .fetch_one(&state)
    .await
    .unwrap();

    return Json(json!({"stautus":201,"data":delete_log})).into_response();
}

#[derive(Deserialize)]
struct DateParams {
    date: Option<String>,
}

async fn get_session_logs(
    State(state): State<PgPool>,
    Query(params): Query<DateParams>,
    Extension(user_id): Extension<String>,
) -> Response {
    let user_id = get_user_id(user_id, &state).await;

    let date = if let Some(d) = params.date {
        match time::Date::parse(&d, &time::format_description::well_known::Iso8601::DATE) {
            Ok(parsed) => parsed,
            Err(_) => return Json(json!({"status": 400, "error": "Invalid date format. Use YYYY-MM-DD"})).into_response(),
        }
    } else {
        time::OffsetDateTime::now_utc().date()
    };

    let logs = session_query(&state, user_id, date).await;

    return Json(json!({"status":201,"data":logs})).into_response();
}

async fn session_query(state: &Pool<Postgres>, user_id: i32, date: time::Date) -> Vec<ExerciseLog> {
    let todays_log = sqlx::query_as!(
        ExerciseLog,
        r#"
        SELECT id,user_id, name, weight_kg, "set", rep, completed_at
        FROM exercise_tracker
        WHERE user_id = $1
          AND completed_at::date = $2
        "#,
        user_id,
        date
    )
    .fetch_all(state)
    .await
    .unwrap();

    todays_log
}

async fn sendmessage(
    State(state): State<PgPool>,
    Extension(user_id): Extension<String>,
) -> Response {
    let client = reqwest::Client::new();

    let telegram_id: i64 = serde_json::from_str::<serde_json::Value>(&user_id)
        .unwrap()
        .get("id")
        .and_then(|v| v.as_i64())
        .unwrap();

    let user_id_num = get_user_id(user_id, &state).await;

    let session_log = session_query(&state, user_id_num, time::OffsetDateTime::now_utc().date()).await;

    let mut exercise_volumes: HashMap<String, f64> = HashMap::new();

    for log in &session_log {
        let name = log.name.as_deref().unwrap_or("unknown").to_string();

        let rep = log.rep.unwrap().to_string();

        let weight = log
            .weight_kg
            .as_ref()
            .and_then(|w| w.to_string().parse::<f64>().ok())
            .unwrap_or(0.0);

        let reps = log.rep.unwrap_or(0) as f64;
        *exercise_volumes.entry(name).or_insert(0.0) += weight * reps;
    }

    let total_session_volume: f64 = exercise_volumes.values().sum();

    let set_lines = session_log
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let weight = log
                .weight_kg
                .as_ref()
                .and_then(|w| w.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            let reps = log.rep.unwrap_or(0) as f64;
            let vol = weight * reps;

            format!(
                "{}. {} | {}kg | set:{} | rep:{} | vol:{}kg",
                i + 1,
                log.name.as_deref().unwrap_or("Unknown"),
                log.weight_kg
                    .as_ref()
                    .map(|w| w.to_string())
                    .unwrap_or_default(),
                log.set.unwrap_or(0),
                log.rep.unwrap_or(0),
                vol,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let exercise_summary = exercise_volumes
        .iter()
        .map(|(name, vol)| format!("{}: {}kg", name, vol))
        .collect::<Vec<_>>()
        .join("\n");
    let text = format!(
        "{}\n\n📊 Exercise Volume:\n{}\n\n💪 Total Session Volume: {}kg",
        set_lines, exercise_summary, total_session_volume
    );
    let res=client.post("https://api.telegram.org/bot8421573811:AAFJ5rCurcQtogGi6x1xK3rRAZx5eZMx3UY/sendMessage").json(&serde_json::json!({
        "chat_id":telegram_id,
        "text":text
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();
    // just analysis this what we done here

    return Json(json!({"status":201,"data":res})).into_response();
}
