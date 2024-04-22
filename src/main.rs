use std::time::Instant;
use usvg::fontdb;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tminimal <in-svg> <out-png>");
        return;
    }

    let tree = {
        let mut opt = usvg::Options::default();
        // Get file's absolute directory.
        opt.resources_dir = std::fs::canonicalize(&args[1])
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();

        let now = Instant::now();
        let svg_data = std::fs::read(&args[1]).unwrap();
        let elapsed = now.elapsed();
        println!("Reading {} took {:.2?}", &args[1], elapsed);

        let now = Instant::now();
        let parsed = usvg::Tree::from_data(&svg_data, &opt, &fontdb).unwrap();
        let elapsed = now.elapsed();
        println!("Parsing took {:.2?}", elapsed);

        parsed
    };

    let now = Instant::now();
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    let elapsed = now.elapsed();
    println!("Rendering took {:.2?}", elapsed);
    pixmap.save_png(&args[2]).unwrap();
}
