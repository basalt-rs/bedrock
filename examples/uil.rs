use std::io;

use basalt_bedrock::{render::typst::PACKET_TEMPLATE, Config};
use tokio::fs::{self, File};

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = fs::read_to_string("./examples/uil.toml").await.unwrap();
    let x = Config::from_str(config, Some("one.toml")).unwrap();
    let out_path = "uil-3.pdf";
    let mut out = File::create(out_path).await.unwrap();
    let bytes = x.generate_pdf_async(&mut out, PACKET_TEMPLATE).await?;

    eprintln!("Wrote {} bytes to {}.", bytes, out_path);

    Ok(())
}
