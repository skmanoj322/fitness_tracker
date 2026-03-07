use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, sqlx::FromRow)]
pub struct ExerciseLog {
    pub id: i32,
    pub user_id: i32,
    pub name: Option<String>,
    pub weight_kg: Option<BigDecimal>,
    pub set: Option<i32>,
    pub rep: Option<i32>,
    pub completed_at: Option<time::OffsetDateTime>,
}

#[derive(Serialize, sqlx::FromRow, Deserialize)]

pub struct NewExerciseLog {
    pub name: String,
    pub weight_kg: Option<BigDecimal>,
    pub set: Option<i32>,
    pub rep: Option<i32>,
}
