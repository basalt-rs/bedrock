use basalt_bedrock::Config;

const FILE_MULTIPLE_COMPLEX: &str = include_str!("./events-multiple-complex.toml");
const FILE_MULTIPLE_SIMPLE: &str = include_str!("./events-multiple-simple.toml");
const FILE_SINGLE_COMPLEX: &str = include_str!("./events-single-complex.toml");
const FILE_SINGLE_SIMPLE: &str = include_str!("./events-single-simple.toml");
const ONE_FILE: &str = include_str!("../examples/one.toml");

#[test]
fn parse_empty_integration_settings() -> miette::Result<()> {
    let config = Config::from_str(ONE_FILE, Some("one.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 0);
    assert_eq!(config.integrations.webhooks.len(), 0);
    Ok(())
}

#[test]
fn parse_multiple_complex() -> miette::Result<()> {
    let config = Config::from_str(FILE_MULTIPLE_COMPLEX, Some("events-multiple-complex.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 0);
    assert_eq!(config.integrations.webhooks.len(), 2);
    Ok(())
}

#[test]
fn parse_multiple_simple() -> miette::Result<()> {
    let config = Config::from_str(FILE_MULTIPLE_SIMPLE, Some("events-multiple-simple.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 0);
    assert_eq!(config.integrations.webhooks.len(), 3);
    Ok(())
}

#[test]
fn parse_single_simple() -> miette::Result<()> {
    let config = Config::from_str(FILE_SINGLE_SIMPLE, Some("events-single-simple.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 1);
    assert_eq!(config.integrations.webhooks.len(), 1);
    Ok(())
}

#[test]
fn parse_single_complex() -> miette::Result<()> {
    let config = Config::from_str(FILE_SINGLE_COMPLEX, Some("events-single-complex.toml"))?;
    assert_eq!(config.integrations.event_handlers.len(), 1);
    assert_eq!(config.integrations.webhooks.len(), 1);
    Ok(())
}
