use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::time::Instant;
use usvg::fontdb;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/tile/{z}/{x}/{y}.png")]
async fn tile(path: web::Path<(u32, u32, u32)>) -> impl Responder {
    let (z, x, y) = path.into_inner();
    HttpResponse::Ok().body(format!("Tile at (z={}, x={}, y={}) requested!", z, x, y))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// https://actix.rs/docs/application#shared-mutable-state

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(tile)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
