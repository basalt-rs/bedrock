use bedrock::Config;

const FILE: &str = include_str!("./events.toml");
const FILE_SINGLE: &str = include_str!("./events-single.toml");

#[test]
fn parse_array() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    assert_eq!(config.integrations.events.len(), 2);
    Ok(())
}

#[test]
fn parse_single() -> miette::Result<()> {
    let config = Config::from_str(FILE_SINGLE, Some("events-single.toml"))?;
    assert_eq!(config.integrations.events.len(), 1);
    Ok(())
}
