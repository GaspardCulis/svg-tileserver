use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
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
    let tree = &data.tree;

    let now = Instant::now();
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let elapsed = now.elapsed();
    println!(
        "Rendering region (z={}, x={}, y={}) took {:.2?}",
        z, x, y, elapsed
    );

    HttpResponse::Ok().body(format!("Tile at (z={}, x={}, y={}) requested!", z, x, y))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const SVG_PATH: &str =
        "C:/Users/a903823/OneDrive - Eviden/Documents/CODE/svguez/public/svg/sil/BT.PCT.svg";

    println!("Starting server...");
    let now = Instant::now();
    let tree = {
        let mut opt = usvg::Options::default();
        // Get file's absolute directory.
        opt.resources_dir = std::fs::canonicalize(&SVG_PATH)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        let svg_data = std::fs::read(&SVG_PATH).unwrap();
        println!("Parsing {}...", &SVG_PATH);
        usvg::Tree::from_data(&svg_data, &opt, &fontdb).unwrap()
    };
    let elapsed = now.elapsed();
    println!("Parsing took {:.2?}", elapsed);

    let state = web::Data::new(AppState { tree });

    println!("Server started!");
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
