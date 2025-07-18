pub mod duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(value: &Duration, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        humantime::format_duration(*value)
            .to_string()
            .serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        humantime::parse_duration(&String::deserialize(de)?).map_err(serde::de::Error::custom)
    }
}

pub mod option_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(value: &Option<Duration>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value
            .map(|value| humantime::format_duration(value).to_string())
            .serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(s) = <Option<String>>::deserialize(de)? {
            humantime::parse_duration(&s)
                .map(Some)
                .map_err(serde::de::Error::custom)
        } else {
            Ok(None)
        }
    }
}
