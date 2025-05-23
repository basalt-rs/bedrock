use bedrock::Config;

const FILE: &str = include_str!("./events.toml");
const JS_FILE: &str = include_str!("./on-score.js");
const TS_FILE: &str = include_str!("./on-complete.ts");

#[test]
fn parse() -> miette::Result<()> {
    Config::from_str(FILE, Some("events.toml"))?;
    Ok(())
}
#[tokio::test]
async fn parse_get_async() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    let mut files = config.events.get_all_files_async().await.unwrap();
    assert_eq!(files.len(), 2);
    files.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(files[0].1, TS_FILE);
    assert_eq!(files[1].1, JS_FILE);
    Ok(())
}
#[test]
fn parse_get_sync() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    let mut files = config.events.get_all_files().unwrap();
    assert_eq!(files.len(), 2);
    files.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(files[0].1, TS_FILE);
    assert_eq!(files[1].1, JS_FILE);
    Ok(())
}
