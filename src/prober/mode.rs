// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Mode {
    #[serde(rename = "poll")]
    Poll,

    #[serde(rename = "push")]
    Push,

    #[serde(rename = "script")]
    Script,

    #[serde(rename = "local")]
    Local,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_deserialize_poll() {
        let mode: Mode = serde_json::from_str("\"poll\"").unwrap();
        assert_eq!(mode, Mode::Poll);
    }

    #[test]
    fn test_mode_deserialize_push() {
        let mode: Mode = serde_json::from_str("\"push\"").unwrap();
        assert_eq!(mode, Mode::Push);
    }

    #[test]
    fn test_mode_deserialize_script() {
        let mode: Mode = serde_json::from_str("\"script\"").unwrap();
        assert_eq!(mode, Mode::Script);
    }

    #[test]
    fn test_mode_deserialize_local() {
        let mode: Mode = serde_json::from_str("\"local\"").unwrap();
        assert_eq!(mode, Mode::Local);
    }

    #[test]
    fn test_mode_deserialize_invalid() {
        let result: Result<Mode, _> = serde_json::from_str("\"unknown\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_mode_serialize() {
        assert_eq!(serde_json::to_string(&Mode::Poll).unwrap(), "\"poll\"");
        assert_eq!(serde_json::to_string(&Mode::Push).unwrap(), "\"push\"");
        assert_eq!(serde_json::to_string(&Mode::Script).unwrap(), "\"script\"");
        assert_eq!(serde_json::to_string(&Mode::Local).unwrap(), "\"local\"");
    }

    #[test]
    fn test_mode_clone_eq() {
        let m = Mode::Poll;
        assert_eq!(m.clone(), m);
        assert_ne!(Mode::Poll, Mode::Push);
    }

    #[test]
    fn test_mode_debug() {
        assert_eq!(format!("{:?}", Mode::Poll), "Poll");
        assert_eq!(format!("{:?}", Mode::Local), "Local");
    }
}
