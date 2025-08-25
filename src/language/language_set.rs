use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::ops::{Deref, DerefMut};

use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};

use crate::language::Version;

use super::{BuiltInLanguage, Language, Syntax};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageSet {
    inner: BTreeSet<Language>,
}

impl LanguageSet {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn get_by_str(&self, raw_name: &str) -> Option<&Language> {
        self.inner.iter().find(|l| l.name() == raw_name)
    }
}

impl Deref for LanguageSet {
    type Target = BTreeSet<Language>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for LanguageSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

struct LanguageMapVisitor;

impl<'de> Visitor<'de> for LanguageMapVisitor {
    type Value = LanguageSet;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a map of languages")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = LanguageSet::new();

        // TODO: Spans or something for better error messages
        while let Some((key, value)) = access.next_entry::<Cow<'_, str>, TomlLanguage>()? {
            let val = match value {
                TomlLanguage::Latest => Language::BuiltIn {
                    language: key.parse().map_err(|()| {
                        serde::de::Error::custom(format!(
                            "Unknown built-in language: '{}'. Known languages: {}",
                            key,
                            BuiltInLanguage::joined_variants()
                        ))
                    })?,
                    version: Version::Latest,
                },
                TomlLanguage::Version(v) => {
                    let language: BuiltInLanguage = key.parse().map_err(|()| {
                        serde::de::Error::custom(format!(
                            "Unknown built-in language: '{}'.  Known languages: {}",
                            key,
                            BuiltInLanguage::joined_variants()
                        ))
                    })?;
                    let version = Version::Specific(v.clone().into());

                    if let Err(versions) = language.has_version(&version) {
                        return Err(serde::de::Error::custom(format!(
                            "Unknown {} version: '{}'.  Known versions: {}",
                            key,
                            v,
                            versions
                                .into_iter()
                                .map(|s| format!("'{}'", s))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )));
                    }

                    Language::BuiltIn { language, version }
                }
                TomlLanguage::Custom {
                    display_name,
                    build,
                    run,
                    workspace,
                    source_file,
                    syntax,
                } => Language::Custom {
                    name: key.clone().into_owned(),
                    display_name: display_name.unwrap_or_else(|| key.clone()).into_owned(),
                    build: build.map(Cow::into_owned),
                    run: run.into_owned(),
                    workspace: workspace.map(Cow::into_owned),
                    syntax: syntax
                        .or_else(|| Syntax::from_string::<M::Error>(key).ok())
                        .unwrap_or_default(),
                    source_file: source_file.into_owned(),
                },
            };

            map.insert(val);
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for LanguageSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LanguageMapVisitor)
    }
}

impl Serialize for LanguageSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for lang in &self.inner {
            match lang {
                Language::BuiltIn {
                    language: name,
                    version: value,
                } => {
                    map.serialize_entry(name.name(), &TomlLanguage::from(value))?;
                }
                Language::Custom {
                    name,
                    display_name,
                    build,
                    workspace,
                    run,
                    source_file,
                    syntax,
                } => {
                    map.serialize_entry(
                        name,
                        &TomlLanguage::Custom {
                            display_name: Some(display_name.into()),
                            build: build.as_ref().map(Into::into),
                            run: run.into(),
                            workspace: workspace.as_ref().map(Cow::from),
                            source_file: source_file.into(),
                            syntax: Some(*syntax),
                        },
                    )?;
                }
            }
        }
        map.end()
    }
}

/// Language as represented in the toml file
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
enum TomlLanguage<'a> {
    #[serde(alias = "*")]
    Latest,
    #[serde(untagged)]
    Version(Cow<'a, str>),
    #[serde(untagged)]
    Custom {
        #[serde(alias = "name")]
        display_name: Option<Cow<'a, str>>,
        build: Option<Cow<'a, str>>,
        run: Cow<'a, str>,
        workspace: Option<Cow<'a, str>>,
        source_file: Cow<'a, str>,
        syntax: Option<Syntax>,
    },
}

impl<'a> From<&'a Version> for TomlLanguage<'a> {
    fn from(value: &'a Version) -> Self {
        match value {
            Version::Latest => TomlLanguage::Latest,
            Version::Specific(v) => TomlLanguage::Version(v.into()),
        }
    }
}
