use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
//use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, process::Command};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, instrument, Level};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
extern crate base64;

#[tokio::main]
async fn main() {
    initialize_logging();

    let cors = CorsLayer::new().allow_origin(Any);

    // build our application with a single route
    let app = Router::new()
        //.route("/", get(|| async { "Hello, World!" }))
        .route("/", get(ledger))
        .route("/hello", get(hello_img))
        .route("/ping", get(|| async { "pong" }))
        .route("/shutdown", get(shutdown))
        // middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    let address = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::debug!("listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/**
 * Initialize and configure logging/tracing to the console window.
 */
fn initialize_logging() {
    // tracing init
    //tracing_subscriber::fmt::init();

    let formatting_layer = tracing_subscriber::fmt::layer();

    let filter = filter::Targets::new()
        .with_target("cashier_server", Level::TRACE)
        // .with_target("tower_http::trace::on_response", Level::TRACE)
        // .with_target("tower_http::trace::on_request", Level::TRACE)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(formatting_layer)
        .with(filter)
        .init();
}

// #[instrument]
async fn hello_img() -> impl IntoResponse {
    // Base64 encoded pixel
    let pixel_encoded = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    // decode pixel
    let decoded = base64::decode(pixel_encoded);

    (
        axum::response::AppendHeaders([(axum::http::header::CONTENT_TYPE, "image/png")]),
        decoded.unwrap(),
    )
}

// #[instrument]
async fn ledger(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if !params.contains_key("command") {
        let mut result: Vec<String> = Vec::new();
        result.push(String::from("No Ledger command sent"));
        return (StatusCode::BAD_REQUEST, Json(result));
    }

    let query = params["command"].as_str();

    let ledger_output = run_ledger(query);

    // split lines
    //let rows: Vec<String> = ledger_output.lines().collect();
    let rows: Vec<String> = ledger_output.lines().map(|x| String::from(x)).collect();

    // convert to Json
    (StatusCode::OK, Json(rows))
}

fn run_ledger(command: &str) -> String {
    // separate command into individual arguments
    let iter = command.split_whitespace();

    let mut ledger = Command::new("ledger");
    ledger.args(iter);

    //let output = ledger.status().expect("process failed to execute");
    let output = ledger.output().expect("failed to execute process");
    //let output = ledger.spawn().expect("ls command failed to start");

    // assert!(output.status.success());
    let result: String;

    if !output.status.success() {
        result = String::from_utf8(output.stderr).unwrap();
        // println!("not success: {}", result);
    } else {
        result = String::from_utf8(output.stdout).unwrap();
    }

    return result;
}

// #[instrument]
async fn shutdown() {
    let msg = "Shutting down on client request...";
    tracing::warn!(msg);
    //panic!("{}", msg);
    // println!("{}", msg);
    std::process::exit(0);
}
