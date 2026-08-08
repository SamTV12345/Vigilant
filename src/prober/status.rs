// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use rmcp::schemars;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, schemars::JsonSchema)]
pub enum Status {
    #[serde(rename = "healthy")]
    Healthy,

    #[serde(rename = "sick")]
    Sick,

    #[serde(rename = "dead")]
    Dead,

    #[serde(rename = "partial")]
    Partial,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            &Status::Healthy => "healthy",
            &Status::Sick => "sick",
            &Status::Dead => "dead",
            &Status::Partial => "partial",
        }
    }

    pub fn as_icon(&self) -> &'static str {
        match self {
            &Status::Dead => "\u{274c}",
            &Status::Sick | &Status::Partial => "\u{26a0}",
            &Status::Healthy => "\u{2705}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str_all_variants() {
        assert_eq!(Status::Healthy.as_str(), "healthy");
        assert_eq!(Status::Sick.as_str(), "sick");
        assert_eq!(Status::Dead.as_str(), "dead");
        assert_eq!(Status::Partial.as_str(), "partial");
    }

    #[test]
    fn test_as_icon_all_variants() {
        assert_eq!(Status::Healthy.as_icon(), "\u{2705}");
        assert_eq!(Status::Sick.as_icon(), "\u{26a0}");
        assert_eq!(Status::Partial.as_icon(), "\u{26a0}");
        assert_eq!(Status::Dead.as_icon(), "\u{274c}");
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Healthy, Status::Healthy);
        assert_ne!(Status::Healthy, Status::Dead);
        assert_ne!(Status::Sick, Status::Partial);
    }

    #[test]
    fn test_status_copy() {
        let s = Status::Sick;
        let copied = s;
        assert_eq!(s, copied);
    }

    #[test]
    fn test_status_debug() {
        assert_eq!(format!("{:?}", Status::Healthy), "Healthy");
        assert_eq!(format!("{:?}", Status::Partial), "Partial");
    }
}
