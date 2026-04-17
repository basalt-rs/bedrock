use serde::{de::IgnoredAny, Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use url::Url;

/// Contains information for all things related to programmability and
/// external integrations in Basalt.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Integrations {
    /// Paths to files that will be executed on server events
    #[serde(
        default,
        alias = "event_handler",
    )]
    #[deprecated(since = "1.1.0", note = "Deprecated in favor of webhooks")]
    pub event_handlers: IgnoredAny,
    #[serde_as(as = "OneOrMany<_>")]
    #[serde(default, alias = "webhook")]
    pub webhooks: Vec<Url>,
}
