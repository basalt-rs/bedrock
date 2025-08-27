use language::{BuiltInLanguage, Language, Syntax, Version};
use miette::Result;

use super::*;
use std::io::Cursor;

const EXAMPLE_ONE_CONTENT: &str = include_str!("../examples/one.toml");

#[test]
fn packets_parse_correctly() -> Result<()> {
    // parse example one
    Config::from_str(EXAMPLE_ONE_CONTENT, Some("Cargo.toml"))?;
    Ok(())
}

#[test]
fn packet_files_parse_correctly() -> Result<()> {
    let mut file = Cursor::new(EXAMPLE_ONE_CONTENT);
    let config = Config::read(&mut file, Some("one.toml"))?;

    assert_eq!(
        Some(&Language::BuiltIn {
            language: BuiltInLanguage::Python3,
            version: Version::Latest
        }),
        config.languages.get_by_str("python3")
    );

    assert_eq!(
        Some(&Language::BuiltIn {
            language: BuiltInLanguage::Java,
            version: Version::Specific("21".into())
        }),
        config.languages.get_by_str("java")
    );

    assert_eq!(
        Some(&Language::Custom {
            name: "ocaml".into(),
            display_name: "ocaml".into(),
            build: Some("ocamlc -o out solution.ml".into()),
            run: "./out".into(),
            source_file: "solution.ml".into(),
            syntax: Syntax::Ocaml,
        }),
        config.languages.get_by_str("ocaml")
    );

    assert_eq!(
        Some(&Language::Custom {
            name: "haskell".into(),
            display_name: "haskell".into(),
            build: Some("ghc solution.hs".into()),
            run: "./solution".into(),
            source_file: "solution.hs".into(),
            syntax: Syntax::Haskell,
        }),
        config.languages.get_by_str("haskell")
    );
    assert_eq!(
        Some(&Language::BuiltIn {
            language: BuiltInLanguage::JavaScript,
            version: Version::Specific("deno:latest".into())
        }),
        config.languages.get_by_str("javascript")
    );

    dbg!(config.hash());

    Ok(())
}

#[tokio::test]
async fn packet_files_parse_correctly_async() -> Result<()> {
    let mut file = Cursor::new(EXAMPLE_ONE_CONTENT);
    let config = Config::read_async(&mut file, Some("one.toml")).await?;
    dbg!(config.hash());
    Ok(())
}

#[test]
fn default_config() {
    let config = Config::default();
    dbg!(config.hash());
}
