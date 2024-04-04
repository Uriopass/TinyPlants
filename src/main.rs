mod db;
mod migrate;

use std::net::SocketAddr;
use std::time::SystemTime;
use axum::response::{Html, IntoResponse};
use axum::{Extension, Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use include_dir::{Dir, include_dir};
use crate::db::Database;

static INDEX: &str = include_str!("../assets/index.html");
pub static MIGRATIONS: Dir = include_dir!("migrations");

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut path = "db.db";
    if std::fs::metadata(path)
        .map(|x| !x.is_file())
        .unwrap_or(true)
        && std::fs::metadata("storage")
        .map(|x| x.is_dir())
        .unwrap_or(false)
    {
        path = "storage/db.db";
    }
    println!("using db: {}", path);
    let db = Database::new(path).expect("could not open db");

    println!(
        "sqlite version: {}",
        db.connection()
            .unwrap()
            .query_row("select sqlite_version();", [], |v| v
                .get::<usize, String>(0))
            .unwrap()
    );

    migrate::migrate(&db.0, &MIGRATIONS).expect("could not run migrations");

    let app = Router::new()
        .route("/", get(root))
        .route("/water", get(water_get))
        .route("/water", post(water_post))
        .route("/history", get(history_get))
        .route("/favicon.png", get(|| async { include_bytes!("../assets/favicon.png") }))
        .route("/cute_plant.png", get(|| async { include_bytes!("../assets/cute_plant.png") }))
        .layer(db)
        ;

    let port = std::env::var("PORT")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(9000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Listening on: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    Html(INDEX)
}

#[derive(serde::Serialize)]
struct WaterResponse {
    timestamp: u64,
}

async fn water_get(Extension(db): Extension<Database>) -> impl IntoResponse {
    let conn = db.connection().unwrap();
    let mut stmt = conn.prepare("SELECT timestamp FROM water ORDER BY timestamp DESC").unwrap();
    let timestamp = stmt.query_row([], |row| row.get(0)).unwrap();
    Json(WaterResponse { timestamp })
}

async fn water_post(Extension(db): Extension<Database>) -> impl IntoResponse {
    let conn = db.connection().unwrap();
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    conn.execute("INSERT INTO water (timestamp) VALUES (?)", [now]).unwrap();
    StatusCode::OK
}

#[derive(serde::Serialize)]
struct HistoryElement {
    timestamp: u64,
}

async fn history_get(Extension(db): Extension<Database>) -> impl IntoResponse {
    let conn = db.connection().unwrap();
    let mut stmt = conn.prepare("SELECT timestamp FROM water ORDER BY timestamp DESC").unwrap();
    let timestamps = stmt.query_map([], |row| row.get(0)).unwrap();
    let timestamps: Vec<HistoryElement> = timestamps
        .map(|x| HistoryElement { timestamp: x.unwrap() })
        .collect();
    Json(timestamps)
}