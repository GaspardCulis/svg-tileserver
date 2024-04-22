use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use std::sync::RwLock;
use std::time::Instant;
use usvg::fontdb;

struct AppState {
    tree: RwLock<usvg::Tree>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/tile/{z}/{x}/{y}.png")]
async fn tile(path: web::Path<(i32, i32, i32)>, data: web::Data<AppState>) -> impl Responder {
    let now = Instant::now();

    let (z, x, y) = path.into_inner();
    let (z, x, y) = (z as f32, x as f32, y as f32);
    let tree = data.tree.read().unwrap();

    let width = 256f32;
    let height = 256f32;
    let scale = z + 1.;
    // TODO: Find magic numbers
    let translate_x = -width * x - width * scale * 1.81;
    let translate_y = -height * y - height * scale * 1.92;

    let _pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::default()
            .post_scale(scale, scale)
            .post_translate(translate_x, translate_y),
        &mut pixmap.as_mut(),
    );
    let elapsed = now.elapsed();
    println!(
        "Rendering region (z={}, x={}, y={}) took {:.2?}",
        z, x, y, elapsed
    );

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .content_type("image/png")
        .body(pixmap.encode_png().unwrap())
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const SVG_PATH: &str =
        "C:/Users/a903823/OneDrive - Eviden/Documents/CODE/svguez/public/svg/sil/BT.PCT.opti.svg";

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

    let state = web::Data::new(AppState {
        tree: RwLock::new(tree),
    });

    println!("Server started!");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(tile)
            .route("/hey", web::get().to(manual_hello))
    })
    .workers(8)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
