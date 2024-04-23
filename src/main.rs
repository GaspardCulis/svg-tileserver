use actix_web::{get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder};
use hex_color::HexColor;
use serde_json::{Result, Value};
use std::borrow::BorrowMut;
use std::sync::RwLock;
use std::time::Instant;
use usvg::fontdb;

struct AppState {
    tree: RwLock<usvg::Tree>, // <- Mutex is necessary to mutate safely across threads
}

unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
    let const_ptr = reference as *const T;
    let mut_ptr = const_ptr as *mut T;
    &mut *mut_ptr
}

#[post("/update")]
async fn update(req_body: String, data: web::Data<AppState>) -> impl Responder {
    const REST_USAGE: &str = "Error, expected JSON object:
{
    id: string, // Node ID to update
    stroke: string // Hex color of the new stroke color
}";

    let parsed: Result<Value> = serde_json::from_str(&req_body);
    if parsed.is_err() {
        return HttpResponse::from_error(parsed.unwrap_err());
    }
    let params = parsed.unwrap();
    if !(params.is_object()) {
        return HttpResponse::build(StatusCode::PRECONDITION_FAILED).body(REST_USAGE);
    }
    let params = params.as_object().unwrap();
    if !(params.contains_key("id") && params.contains_key("stroke")) {
        return HttpResponse::build(StatusCode::PRECONDITION_FAILED).body(REST_USAGE);
    }
    let (id, stroke) = (params.get("id").unwrap(), params.get("stroke").unwrap());
    if !(id.is_string() && stroke.is_string()) {
        return HttpResponse::build(StatusCode::PRECONDITION_FAILED).body(REST_USAGE);
    }
    let (id, stroke) = (id.as_str().unwrap(), stroke.as_str().unwrap());

    let stroke = HexColor::parse(stroke);
    if stroke.is_err() {
        return HttpResponse::build(StatusCode::PRECONDITION_FAILED).body(REST_USAGE);
    }
    let stroke = stroke.unwrap();
    let paint = usvg::Paint::Color(usvg::Color {
        red: stroke.r,
        green: stroke.g,
        blue: stroke.b,
    });

    let tree = data.tree.write().unwrap();
    let mut node = tree.node_by_id(id).unwrap();
    let node_mut = node.borrow_mut();
    match node_mut {
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

    HttpResponse::Ok().body(req_body)
}

#[get("/tile/{z}/{x}/{y}.png")]
async fn tile(path: web::Path<(i32, i32, i32)>, data: web::Data<AppState>) -> impl Responder {
    let now = Instant::now();

    let (z, x, y) = path.into_inner();
    let (z, x, y) = (z as f32, x as f32, y as f32);
    let tree = data.tree.read().unwrap();

    let width = 1024f32;
    let height = 1024f32;
    let scale = z + 1.;
    // TODO: Find magic numbers
    let translate_x = -width * x - width * scale / 2.;
    let translate_y = -height * y - height * scale / 2.;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const SVG_PATH: &str =
        "C:/Users/a903823/OneDrive - Eviden/Documents/CODE/svguez/public/svg/elecgeo/ELECGEO.PCT.opti.svg";

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
            .service(update)
            .service(tile)
    })
    .workers(8)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
