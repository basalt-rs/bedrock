use std::io;

use basalt_bedrock::render::typst::{LOGIN_TEMPLATE, PACKET_TEMPLATE};

fn main() -> io::Result<()> {
    let config = std::fs::read_to_string("./examples/render-test.toml").unwrap();

    let x = basalt_bedrock::Config::from_str(config, Some("one.toml")).unwrap();

    let path = "packet.pdf";
    let mut file = std::fs::File::create(path).unwrap();
    let bytes = x.generate_pdf(&mut file, PACKET_TEMPLATE)?;
    eprintln!("Wrote {} bytes to {}.", bytes, path);

    let path = "logins.pdf";
    let mut file = std::fs::File::create(path).unwrap();
    let bytes = x.generate_pdf(&mut file, LOGIN_TEMPLATE)?;
    eprintln!("Wrote {} bytes to {}.", bytes, path);

    Ok(())
}
