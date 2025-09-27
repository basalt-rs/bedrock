use std::path::{Path, PathBuf};

/// Get the cmarker plugin for typst
#[cfg(feature = "render")]
fn fetch_typst_plugin(name: &str, version: &str, files: &[&str]) {
    use flate2::read::GzDecoder;
    use reqwest::blocking as reqwest;
    use tar::Archive;

    use std::fs::File;
    use std::io::BufWriter;

    let url = format!(
        "https://packages.typst.org/preview/{}-{}.tar.gz",
        name, version
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .join("typst-plugins")
        .join(name);

    let response = reqwest::get(url).expect("fetching cmarker plugin");
    let tar = GzDecoder::new(response);
    let mut archive = Archive::new(tar);
    std::fs::create_dir_all(&out_dir).unwrap();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        for path in files {
            if entry.path().unwrap() == Path::new(path) {
                let path = out_dir.join(path);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut file = BufWriter::new(File::create(&path).unwrap());
                std::io::copy(&mut entry, &mut file).unwrap_or_else(|e| {
                    panic!("Error while unpacking {}: {:?}", path.display(), e)
                });
            }
        }
    }
}

macro_rules! add_typst_plugin {
    ($files: expr, $name: literal, $version: literal, [$($path: literal),+$(,)?]) => {
        fetch_typst_plugin($name, $version, &[$($path),+]);
        $($files.push(concat!($name, "/", $path));)+
    };
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    #[cfg(feature = "render")]
    {
        use std::io::Write;
        let mut files = Vec::new();
        add_typst_plugin!(files, "cmarker", "0.1.6", ["plugin.wasm", "lib.typ"]);
        add_typst_plugin!(
            files,
            "mitex",
            "0.2.5",
            [
                "mitex.wasm",
                "mitex.typ",
                "lib.typ",
                "specs/latex/standard.typ",
                "specs/mod.typ",
                "specs/prelude.typ",
            ]
        );

        let include =
            PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("typst-plugins/include.rs");
        let mut out = std::io::BufWriter::new(std::fs::File::create(include).unwrap());
        write!(out, "&[").expect("can't fail");
        for file in files {
            write!(
                out,
                "({0:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/typst-plugins/\", {0:?}))),",
                file
            )
            .expect("can't fail");
        }
        write!(out, "]").expect("can't fail");
    }
}
