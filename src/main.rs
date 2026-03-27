use axum::{Router, http::Method, routing::get};
// use sqlx::{PgPool, Pool};

use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer}; // Add this import

use telegram_bot::router::user_workout_log;

use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let conn_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let pool: PgPool = PgPool::connect(&conn_url).await.unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "your_project_name=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new()
        // Allow requests from any origin (important for ngrok/local testing)
        .allow_origin(Any)
        // Allow the headers you are sending from Next.js
        .allow_headers(Any)
        // Allow the methods you need
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS]);

    let app = Router::new()
        .merge(user_workout_log())
        .route("/get", get(|| async { "Helloworld" }))
        .with_state(pool)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
