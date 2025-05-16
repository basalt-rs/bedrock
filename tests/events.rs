use bedrock::Config;

const FILE: &str = include_str!("../examples/events.toml");
const JS_FILE: &str = include_str!("../examples/on_score.js");

#[test]
fn parse() -> miette::Result<()> {
    Config::from_str(FILE, Some("events.toml"))?;
    Ok(())
}

#[tokio::test]
async fn parse_get() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    dbg!(config.hash());
    let files = config.events.get_all_files().await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], JS_FILE);
    Ok(())
}
