use axum::{
    routing::post,
    Router
};

#[tokio::main]
async fn main() {
    let node = Router::new().route("/api/tx", post(|| async { println!("hello tx") }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
    axum::serve(listener, node).await.unwrap()
}