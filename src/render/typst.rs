use std::process::Stdio;

use thiserror::Error;

use crate::Config;

/// Files copied for plugin support.  These are fetched in the build.rs
const PLUGIN_FILES: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/typst-plugins/include.rs"));

pub const PACKET_TEMPLATE: &str = include_str!("../../data/template.typ");
pub const LOGIN_TEMPLATE: &str = include_str!("../../data/login-template.typ");

#[derive(Debug, Error)]
pub enum PdfGenerationError {
    #[error("`typst` command not in system path")]
    MissingTypstCommand,
    #[error("Typst exited unsuccessfully")]
    TypstError,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl From<PdfGenerationError> for std::io::Error {
    fn from(value: PdfGenerationError) -> Self {
        match value {
            PdfGenerationError::MissingTypstCommand => std::io::Error::other(value),
            PdfGenerationError::TypstError => std::io::Error::other(value),
            PdfGenerationError::IoError(error) => error,
        }
    }
}

pub(crate) fn generate_pdf(
    config: &Config,
    writer: &mut impl std::io::Write,
    template: impl AsRef<str>,
) -> Result<u64, PdfGenerationError> {
    use std::fs;

    let tmp = tempfile::tempdir()?;

    let config_path = tmp.path().join("config.json");
    let mut out = std::io::BufWriter::new(fs::File::create_new(&config_path)?);
    serde_json::to_writer_pretty(&mut out, config).map_err(std::io::Error::other)?;
    drop(out); // flush the writer

    for (path, content) in PLUGIN_FILES {
        let path = tmp.path().join(path);
        fs::create_dir_all(path.parent().expect("We always have at least the tempdir"))?;
        fs::write(path, content)?;
    }

    let template_path = tmp.path().join("template.typ");
    fs::write(&template_path, template.as_ref())?;

    // typst compile --root <tmp> --input "config=config.json" template.typ -
    let child = std::process::Command::new("typst")
        .arg("compile")
        .current_dir(tmp.path())
        .arg("--root") // --root <tmp>
        .arg(tmp.path())
        .args(["--input", "config=config.json"])
        .arg("template.typ") // input file
        .arg("-") // output file
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            return Err(match e.kind() {
                std::io::ErrorKind::NotFound => PdfGenerationError::MissingTypstCommand,
                _ => PdfGenerationError::IoError(e),
            });
        }
    };

    let mut stdout = child.stdout.take().expect("We're piping stdout");
    let bytes = std::io::copy(&mut stdout, writer)?;

    let status = child.wait()?;

    if status.success() {
        drop(tmp);
        Ok(bytes)
    } else {
        drop(tmp);
        Err(PdfGenerationError::TypstError)
    }
}

#[cfg(feature = "tokio")]
pub(crate) async fn generate_pdf_async<'a, W, T>(
    config: &'a Config,
    writer: &'a mut W,
    template: T,
) -> Result<u64, PdfGenerationError>
where
    W: tokio::io::AsyncWrite + Unpin,
    T: AsRef<str> + Send + Sync + 'static,
{
    use tokio::{fs, task::JoinSet};

    let tmp = tmpdir::TmpDir::new("bedrock").await?;
    let tmp_path = tmp.to_path_buf();

    let config_path = tmp.as_ref().join("config.json");
    let config = serde_json::to_string(config).map_err(std::io::Error::other)?;
    fs::write(&config_path, config).await?;

    let mut js = JoinSet::new();
    for (path, content) in PLUGIN_FILES {
        let path = tmp_path.join(path);
        js.spawn(async move {
            fs::create_dir_all(path.parent().expect("We always have at least the tempdir")).await?;
            fs::write(path, content).await?;
            Ok::<_, PdfGenerationError>(())
        });
    }

    js.spawn(async move {
        let template_path = tmp_path.join("template.typ");
        fs::write(&template_path, template.as_ref()).await?;
        Ok::<_, PdfGenerationError>(())
    });

    // NOTE: we're not using `join_all` to not allocate the vec
    while let Some(x) = js.join_next().await {
        x.map_err(std::io::Error::from)??;
    }

    // typst compile --root <tmp> --input "config=config.json" template.typ -
    let child = tokio::process::Command::new("typst")
        .arg("compile")
        .current_dir(tmp.as_ref())
        .arg("--root") // --root <tmp>
        .arg(tmp.as_ref())
        .args(["--input", "config=config.json"])
        .arg("template.typ") // input file
        .arg("-") // output file
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            return Err(match e.kind() {
                std::io::ErrorKind::NotFound => PdfGenerationError::MissingTypstCommand,
                _ => PdfGenerationError::IoError(e),
            });
        }
    };

    let mut stdout = child.stdout.take().expect("We're piping stdout");
    let bytes = tokio::io::copy(&mut stdout, writer).await?;

    let status = child.wait().await?;

    if status.success() {
        drop(tmp);
        Ok(bytes)
    } else {
        drop(tmp);
        Err(PdfGenerationError::TypstError)
    }
}
