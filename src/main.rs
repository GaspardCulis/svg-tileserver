use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::time::Instant;
use usvg::fontdb;

struct AppState {
    tree: usvg::Tree, // <- Mutex is necessary to mutate safely across threads
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/tile/{z}/{x}/{y}.png")]
async fn tile(path: web::Path<(u32, u32, u32)>, data: web::Data<AppState>) -> impl Responder {
    let (z, x, y) = path.into_inner();
    HttpResponse::Ok().body(format!("Tile at (z={}, x={}, y={}) requested!", z, x, y))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// https://actix.rs/docs/application#shared-mutable-state

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const svg_path: &str =
        "C:/Users/a903823/OneDrive - Eviden/Documents/CODE/svguez/public/svg/sil/BT.PCT.svg";

    let tree = {
        let mut opt = usvg::Options::default();
        // Get file's absolute directory.
        opt.resources_dir = std::fs::canonicalize(&svg_path)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        let svg_data = std::fs::read(&svg_path).unwrap();
        usvg::Tree::from_data(&svg_data, &opt, &fontdb).unwrap()
    };

    let state = web::Data::new(AppState { tree });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(tile)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
