// SPDX-License-Identifier: MIT

use std::convert::TryInto;
use std::fmt;

use serde::de::{Deserializer, Error, Unexpected, Visitor};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(HumanNumberVisitor)
}

struct HumanNumberVisitor;

impl<'de> Visitor<'de> for HumanNumberVisitor {
    type Value = Option<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsigned number or quoted human-readable unsigned number")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.try_into()
            .map_err(Error::custom)
            .and_then(|v| self.visit_u64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Some(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let v = v.trim();

        if v.is_empty() {
            return Err(Error::invalid_value(
                Unexpected::Str(v),
                &"non-empty string",
            ));
        }

        // Using `bytes` instead of `chars` here because `split_at` takes a byte offset
        match v
            .bytes()
            .map(|byte| byte as char)
            .position(|ch| !(ch.is_numeric() || ch == '.'))
        {
            Some(unit_index) => {
                let (value, unit) = v.split_at(unit_index);
                let value: f64 = value.parse().map_err(Error::custom)?;

                let multiple: u64 = match unit.trim_start() {
                    "KB" | "kB" => 1000,
                    "MB" => 1000 * 1000,
                    "GB" => 1000 * 1000 * 1000,
                    "TB" => 1000 * 1000 * 1000 * 1000,

                    "KiB" => 1024,
                    "MiB" => 1024 * 1024,
                    "GiB" => 1024 * 1024 * 1024,
                    "TiB" => 1024 * 1024 * 1024 * 1024,

                    unit => {
                        return Err(Error::invalid_value(
                            Unexpected::Str(unit),
                            &"known unit of measurement",
                        ))
                    }
                };
                let value = value * (multiple as f64);

                Ok(Some(value.round() as u64))
            }
            None => {
                // No unit found, interpret as raw number
                v.parse().map(Some).map_err(Error::custom)
            }
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

#[cfg(test)]
mod test {
    use serde::de::value::Error;
    use serde::de::IntoDeserializer;

    use super::deserialize;

    #[test]
    fn test_deserialize() {
        fn check_result(input: &str, value: u64) {
            assert_eq!(
                deserialize(input.into_deserializer()),
                Ok::<_, Error>(Some(value))
            );
        }

        check_result("1234", 1234);
        check_result("1234 ", 1234);
        check_result(" 1234", 1234);
        check_result(" 1234 ", 1234);

        check_result("1234kB", 1234 * 1000);
        check_result("1234KB", 1234 * 1000);
        check_result("1234MB", 1234 * 1000 * 1000);
        check_result("1234GB", 1234 * 1000 * 1000 * 1000);
        check_result("1234TB", 1234 * 1000 * 1000 * 1000 * 1000);

        check_result("1234KiB", 1234 * 1024);
        check_result("1234MiB", 1234 * 1024 * 1024);
        check_result("1234GiB", 1234 * 1024 * 1024 * 1024);
        check_result("1234TiB", 1234 * 1024 * 1024 * 1024 * 1024);

        check_result("1234 kB", 1234 * 1000);
        check_result("1234 KB", 1234 * 1000);
        check_result("1234 MB", 1234 * 1000 * 1000);
        check_result("1234 GB", 1234 * 1000 * 1000 * 1000);
        check_result("1234 TB", 1234 * 1000 * 1000 * 1000 * 1000);

        check_result("1234 KiB", 1234 * 1024);
        check_result("1234 MiB", 1234 * 1024 * 1024);
        check_result("1234 GiB", 1234 * 1024 * 1024 * 1024);
        check_result("1234 TiB", 1234 * 1024 * 1024 * 1024 * 1024);
    }
}
