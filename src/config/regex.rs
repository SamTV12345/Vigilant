// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt;
use std::ops::Deref;

use regex;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl Deref for Regex {
    type Target = regex::Regex;

    fn deref(&self) -> &regex::Regex {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Regex {
    fn deserialize<D>(de: D) -> Result<Regex, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = Regex;

            fn expecting(&self, format: &mut fmt::Formatter) -> fmt::Result {
                format.write_str("a regular expression pattern")
            }

            fn visit_str<E: Error>(self, value: &str) -> Result<Regex, E> {
                regex::Regex::new(value)
                    .map(Regex)
                    .map_err(|err| E::custom(err.to_string()))
            }
        }

        de.deserialize_str(RegexVisitor)
    }
}

impl Serialize for Regex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Notice: ignore Regex serialization here, as it is not used in templates (which \
        //   serialization is used for in Vigil).
        serializer.serialize_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_valid_regex() {
        let re: Regex = serde_json::from_str("\"^hello.*\"").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("nope"));
    }

    #[test]
    fn test_deserialize_valid_case_insensitive() {
        let re: Regex = serde_json::from_str("\"(?i)hello\"").unwrap();
        assert!(re.is_match("HELLO"));
    }

    #[test]
    fn test_deserialize_invalid_regex() {
        let result: Result<Regex, _> = serde_json::from_str("\"(\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_empty_pattern() {
        let re: Regex = serde_json::from_str("\"\"").unwrap();
        assert!(re.is_match("anything"));
    }

    #[test]
    fn test_serialize_always_null() {
        let re: Regex = serde_json::from_str("\"test\"").unwrap();
        let serialized = serde_json::to_string(&re).unwrap();
        assert_eq!(serialized, "null");
    }
}
