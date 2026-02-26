use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

use miette::NamedSource;
use serde::{
    de::{
        value::{
            EnumAccessDeserializer, MapAccessDeserializer, SeqAccessDeserializer, UnitDeserializer,
        },
        DeserializeOwned, IntoDeserializer, MapAccess, Visitor,
    },
    Deserialize, Serialize,
};

use crate::ConfigReadError;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[non_exhaustive]
pub struct Deser;
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[non_exhaustive]
pub struct Raw;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct RawOrImport<T, Mode = Deser>(T, PhantomData<Mode>)
where
    Mode: Sized;

impl<T, Mode> RawOrImport<T, Mode> {
    pub fn inner(&self) -> &T {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, Mode> Deref for RawOrImport<T, Mode> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, Mode> DerefMut for RawOrImport<T, Mode> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, Mode> From<T> for RawOrImport<T, Mode> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(deny_unknown_fields)]
struct Import {
    import: PathBuf,
}

struct RoiVisitor<T, Mode>(PhantomData<(T, Mode)>);

impl<'de, T> Visitor<'de> for RoiVisitor<T, Deser>
where
    T: DeserializeOwned,
{
    type Value = RawOrImport<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "RawOrImport")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if map.size_hint().is_some_and(|hint| hint != 1) {
            return Ok(T::deserialize(MapAccessDeserializer::new(map))?.into());
        }

        /// Map access that has been peeked for one key
        struct PeekedMapAccess<A>(Option<String>, A);

        impl<'de, A> MapAccess<'de> for PeekedMapAccess<A>
        where
            A: MapAccess<'de>,
        {
            type Error = A::Error;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: serde::de::DeserializeSeed<'de>,
            {
                match self.0.take() {
                    Some(v) => seed.deserialize(v.into_deserializer()).map(Some),
                    None => self.1.next_key_seed(seed),
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                self.1.next_value_seed(seed)
            }
        }
        let key: Option<String> = map.next_key()?;
        let t = match key {
            // if the first key is `import`, we can assume that the structure needs to be
            // an import.
            Some(key) if key == "import" => {
                // use Import::deserialize here, rather than just `next_key` so we can be
                // sure that the structure is correct (and get decent error messages)
                let import = Import::deserialize(MapAccessDeserializer::new(PeekedMapAccess(
                    Some(key),
                    map,
                )))?;

                // TODO: Make this us a relative import path
                let content =
                    std::fs::read_to_string(&import.import).map_err(serde::de::Error::custom)?;

                toml_edit::de::from_str(&content).map_err(|e| {
                    serde::de::Error::custom(ConfigReadError::malformed(
                        NamedSource::new(import.import.display().to_string(), content),
                        e,
                    ))
                })?
            }
            // any other key needs to be deserialized as the expected structure using
            // `PeekedMapAccess` to yield the key that we already consumed
            Some(key) => {
                T::deserialize(MapAccessDeserializer::new(PeekedMapAccess(Some(key), map)))?
            }
            // If the structure is empty, use deserialize on the `T` so that defaults can
            // be provided
            None => T::deserialize(MapAccessDeserializer::new(map))?,
        };

        Ok(t.into())
    }
}

impl<'de, T> Deserialize<'de> for RawOrImport<T, Deser>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RoiVisitor::<_, Deser>(PhantomData))
    }
}

macro_rules! visit_impl {
    ($fn: ident($ty: ty)) => {
        fn $fn<E>(self, v: $ty) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(S::deserialize(v.into_deserializer())?.into())
        }
    };
    ($($fn: ident($ty: ty))*) => {
        $(visit_impl!($fn($ty));)*
    };
}

impl<'de, S> Visitor<'de> for RoiVisitor<S, Raw>
where
    S: FromStr + Deserialize<'de>,
    S::Err: std::fmt::Display,
{
    type Value = RawOrImport<S, Raw>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "RawOrImport")
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let import = Import::deserialize(MapAccessDeserializer::new(map))?;
        // TODO: Make this us a relative import path
        let content = std::fs::read_to_string(&import.import).map_err(serde::de::Error::custom)?;

        Ok(S::from_str(&content)
            .map_err(serde::de::Error::custom)?
            .into())
    }

    visit_impl!(
        visit_bool(bool)
        visit_i8(i8)
        visit_i16(i16)
        visit_i32(i32)
        visit_i64(i64)
        visit_i128(i128)
        visit_u8(u8)
        visit_u16(u16)
        visit_u32(u32)
        visit_u64(u64)
        visit_u128(u128)
        visit_f32(f32)
        visit_f64(f64)
        visit_str(&str)
        visit_bytes(&[u8])
    );

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(S::deserialize(UnitDeserializer::new())?.into())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(S::deserialize(deserializer)?.into())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(S::deserialize(UnitDeserializer::new())?.into())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(S::deserialize(deserializer)?.into())
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        Ok(S::deserialize(SeqAccessDeserializer::new(seq))?.into())
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        Ok(S::deserialize(EnumAccessDeserializer::new(data))?.into())
    }
}

impl<'de, S> Deserialize<'de> for RawOrImport<S, Raw>
where
    S: FromStr + Deserialize<'de>,
    S::Err: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RoiVisitor::<_, Raw>(PhantomData))
    }
}
