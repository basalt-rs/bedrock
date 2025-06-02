use bedrock::Config;

const FILE: &str = include_str!("./events.toml");

#[test]
fn parse() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    assert_eq!(config.integrations.events.len(), 2);
    Ok(())
}
