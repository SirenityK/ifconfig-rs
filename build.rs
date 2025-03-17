use lightningcss::{
    bundler::{Bundler, FileProvider},
    printer::PrinterOptions,
    stylesheet::ParserOptions,
};
use std::{fs, path::Path};

fn main() {
    let fs = FileProvider::new();
    let mut bundler = Bundler::new(&fs, None, ParserOptions::default());
    let stylesheet = bundler.bundle(Path::new("src/styles.css")).unwrap();
    let mut options = PrinterOptions::default();
    options.minify = true;
    fs::write(
        "src/styles.min.css",
        stylesheet.to_css(options).unwrap().code,
    )
    .unwrap();
}
