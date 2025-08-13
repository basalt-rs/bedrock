use bedrock::Config;

const FILE: &str = include_str!("./events.toml");
const FILE_SINGLE: &str = include_str!("./events-single.toml");
const ONE_FILE: &str = include_str!("../examples/one.toml");

#[test]
fn parse_empty_integration_settings() -> miette::Result<()> {
    let config = Config::from_str(ONE_FILE, Some("one.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 0);
    assert_eq!(config.integrations.webhooks.len(), 0);
    Ok(())
}

#[test]
fn parse_multiple() -> miette::Result<()> {
    let config = Config::from_str(FILE, Some("events.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 2);
    assert_eq!(config.integrations.webhooks.len(), 2);
    Ok(())
}

#[test]
fn parse_single() -> miette::Result<()> {
    let config = Config::from_str(FILE_SINGLE, Some("events-single.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 1);
    assert_eq!(config.integrations.webhooks.len(), 1);
    Ok(())
}
