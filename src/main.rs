use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use hex_color::{Display, HexColor};
use serde::Deserialize;
use std::sync::RwLock;
use std::time::Instant;
use usvg::fontdb;

struct AppState {
    tree: RwLock<usvg::Tree>, // <- Mutex is necessary to mutate safely across threads
    tile_size: u32,
}

#[derive(Deserialize)]
struct UpdateParams {
    id: String,
    stroke: String,
}

unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
    let const_ptr = reference as *const T;
    let mut_ptr = const_ptr as *mut T;
    &mut *mut_ptr
}

#[post("/update")]
async fn update(params: web::Json<UpdateParams>, data: web::Data<AppState>) -> impl Responder {
    let UpdateParams { id, stroke } = params.0;

    let stroke = HexColor::parse(&stroke).unwrap();
    let paint = usvg::Paint::Color(usvg::Color {
        red: stroke.r,
        green: stroke.g,
        blue: stroke.b,
    });

    let now = Instant::now();
    let tree = data.tree.write().unwrap();
    let node = tree.node_by_id(id.as_str()).unwrap();
    match node {
        usvg::Node::Path(e) => {
            let stroke = e.stroke.as_ref().unwrap();

            unsafe {
                let mut_stroke = very_bad_function(stroke);
                mut_stroke.set_paint(paint);
            }

            println!("Stroke updated");
        }
        _ => {}
    }

    let elapsed = now.elapsed();
    HttpResponse::Ok()
        .status(StatusCode::OK)
        .content_type("text/plain")
        .body(
            format!(
                "Successfully updated node #{} stroke to {} in {:.2?}",
                id,
                Display::new(stroke),
                elapsed
            )
            .to_string(),
        )
}

#[get("/tile/{z}/{x}/{y}.png")]
async fn tile(path: web::Path<(i32, i32, i32)>, data: web::Data<AppState>) -> impl Responder {
    let now = Instant::now();

    let (z, x, y) = path.into_inner();
    let (z, x, y) = (z as f32, x as f32, y as f32);
    let tree = data.tree.read().unwrap();

    let width = data.tile_size as f32;
    let height = width;
    let scale = 2f32.powf(z);
    let translate_x = -width * x;
    let translate_y = -height * y;

    let _pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::default()
            .pre_translate(-width / 2., -height / 2.)
            .post_scale(scale, scale)
            .post_translate(translate_x, translate_y),
        &mut pixmap.as_mut(),
    );

    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(255, 255, 0, 255);

    let stroke = tiny_skia::Stroke::default();
    let mut path_builder = tiny_skia::PathBuilder::new();
    path_builder.line_to(width, 0.);
    let path = path_builder.finish().unwrap();
    pixmap.stroke_path(
        &path,
        &paint,
        &stroke,
        tiny_skia::Transform::default(),
        None,
    );
    let mut path_builder = tiny_skia::PathBuilder::new();
    path_builder.line_to(0., height);
    let path = path_builder.finish().unwrap();
    pixmap.stroke_path(
        &path,
        &paint,
        &stroke,
        tiny_skia::Transform::default(),
        None,
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path of the SVG that should be served
    #[arg()]
    svg_path: String,

    /// The size in pixels of a PNG tile
    #[arg(short, long, default_value_t = 256)]
    tile_size: u32,

    /// The port to start the server on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// The size in pixels of a PNG tile
    #[arg(short, long, default_value = "127.0.0.1")]
    bind_address: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Args {
        svg_path,
        tile_size,
        port,
        bind_address,
    } = Args::parse();

    println!("Starting server...");
    let now = Instant::now();
    let tree = {
        let mut opt = usvg::Options::default();
        // Get file's absolute directory.
        opt.resources_dir = std::fs::canonicalize(&svg_path)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        let svg_data = std::fs::read(&svg_path).unwrap();
        println!("Parsing {}...", &svg_path);
        usvg::Tree::from_data(&svg_data, &opt, &fontdb).unwrap()
    };
    let elapsed = now.elapsed();
    println!("Parsing took {:.2?}", elapsed);

    let state = web::Data::new(AppState {
        tree: RwLock::new(tree),
        tile_size,
    });

    println!("Server started!");
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(update)
            .service(tile)
    })
    .workers(8)
    .bind((bind_address, port))?
    .run()
    .await
}
