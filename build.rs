use std::{fs, path::Path};

fn find_css_file() -> Result<String, &'static str> {
    let paths = fs::read_dir("html/dist/_astro")
        .expect("Unable to read dist directory. Build the project first.");
    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "css" {
                    return Ok(path.to_str().unwrap().to_string());
                }
            }
        }
    }
    return Err("No css file found");
}

fn main() {
    let css_file = find_css_file().unwrap();
    let css_file = Path::new(&css_file);
    let dest = Path::new("src/styles.min.css");
    fs::copy(css_file, dest).expect("Unable to copy css file");
}
