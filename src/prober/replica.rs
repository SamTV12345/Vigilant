// Vigil
//
// Microservices Status Page
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::{Host, Url};

#[derive(Serialize, Debug, Clone)]
pub enum ReplicaURL {
    ICMP(String),
    TCP(String, u16),
    SSH(String, u16),
    HTTP(String),
    HTTPS(String),
}

impl ReplicaURL {
    pub fn parse_from(raw_url: &str) -> Result<ReplicaURL, ()> {
        match Url::parse(raw_url) {
            Ok(url) => match url.scheme() {
                "icmp" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), None, None) => Ok(ReplicaURL::ICMP(Self::host_string(host))),
                    _ => Err(()),
                },
                "tcp" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), Some(port), None) => {
                        Ok(ReplicaURL::TCP(Self::host_string(host), port))
                    }
                    _ => Err(()),
                },
                "ssh" => match (url.host(), url.port(), url.path_segments()) {
                    (Some(host), Some(port), None) => {
                        Ok(ReplicaURL::SSH(Self::host_string(host), port))
                    }
                    _ => Err(()),
                },
                "http" => Ok(ReplicaURL::HTTP(url.into())),
                "https" => Ok(ReplicaURL::HTTPS(url.into())),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }

    pub fn host_string(host: Host<&str>) -> String {
        // Convert internal host value into string. This is especially useful for IPv6 addresses, \
        //   which we need returned in '::1' format; as they would otherwise be returned in \
        //   '[::1]' format using built-in top-level 'to_string()' method on the 'Host' trait. The \
        //   underlying address parser does not accept IPv6 addresses formatted as '[::1]', so \
        //   this seemingly overkill processing is obviously needed.
        match host {
            Host::Domain(domain_value) => domain_value.to_string(),
            Host::Ipv4(ipv4_value) => ipv4_value.to_string(),
            Host::Ipv6(ipv6_value) => ipv6_value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ReplicaURL::parse_from tests ---

    #[test]
    fn test_parse_icmp_domain() {
        let url = ReplicaURL::parse_from("icmp://example.com").unwrap();
        match url {
            ReplicaURL::ICMP(host) => assert_eq!(host, "example.com"),
            _ => panic!("expected ICMP variant"),
        }
    }

    #[test]
    fn test_parse_icmp_ipv4() {
        let url = ReplicaURL::parse_from("icmp://192.168.1.1").unwrap();
        match url {
            ReplicaURL::ICMP(host) => assert_eq!(host, "192.168.1.1"),
            _ => panic!("expected ICMP variant"),
        }
    }

    #[test]
    fn test_parse_icmp_ipv6() {
        let url = ReplicaURL::parse_from("icmp://[::1]").unwrap();
        match url {
            ReplicaURL::ICMP(host) => assert_eq!(host, "::1"),
            _ => panic!("expected ICMP variant"),
        }
    }

    #[test]
    fn test_parse_icmp_with_port_fails() {
        assert!(ReplicaURL::parse_from("icmp://example.com:80").is_err());
    }

    #[test]
    fn test_parse_icmp_with_path_fails() {
        assert!(ReplicaURL::parse_from("icmp://example.com/path").is_err());
    }

    #[test]
    fn test_parse_tcp() {
        let url = ReplicaURL::parse_from("tcp://example.com:8080").unwrap();
        match url {
            ReplicaURL::TCP(host, port) => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 8080);
            }
            _ => panic!("expected TCP variant"),
        }
    }

    #[test]
    fn test_parse_tcp_without_port_fails() {
        assert!(ReplicaURL::parse_from("tcp://example.com").is_err());
    }

    #[test]
    fn test_parse_tcp_with_path_fails() {
        assert!(ReplicaURL::parse_from("tcp://example.com:80/path").is_err());
    }

    #[test]
    fn test_parse_ssh() {
        let url = ReplicaURL::parse_from("ssh://example.com:22").unwrap();
        match url {
            ReplicaURL::SSH(host, port) => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 22);
            }
            _ => panic!("expected SSH variant"),
        }
    }

    #[test]
    fn test_parse_ssh_without_port_fails() {
        assert!(ReplicaURL::parse_from("ssh://example.com").is_err());
    }

    #[test]
    fn test_parse_http() {
        let url = ReplicaURL::parse_from("http://example.com/path?q=1").unwrap();
        match url {
            ReplicaURL::HTTP(s) => assert!(s.contains("example.com")),
            _ => panic!("expected HTTP variant"),
        }
    }

    #[test]
    fn test_parse_https() {
        let url = ReplicaURL::parse_from("https://example.com").unwrap();
        match url {
            ReplicaURL::HTTPS(s) => assert!(s.contains("example.com")),
            _ => panic!("expected HTTPS variant"),
        }
    }

    #[test]
    fn test_parse_unknown_scheme_fails() {
        assert!(ReplicaURL::parse_from("ftp://example.com").is_err());
        assert!(ReplicaURL::parse_from("unknown://example.com").is_err());
    }

    #[test]
    fn test_parse_invalid_url_fails() {
        assert!(ReplicaURL::parse_from("not-a-url").is_err());
        assert!(ReplicaURL::parse_from("").is_err());
    }

    #[test]
    fn test_parse_tcp_ipv6_bracketed() {
        let url = ReplicaURL::parse_from("tcp://[::1]:9090").unwrap();
        match url {
            ReplicaURL::TCP(host, port) => {
                assert_eq!(host, "::1");
                assert_eq!(port, 9090);
            }
            _ => panic!("expected TCP variant"),
        }
    }

    // --- ReplicaURL::host_string tests ---

    #[test]
    fn test_host_string_domain() {
        use url::Host;
        assert_eq!(
            ReplicaURL::host_string(Host::Domain("example.com")),
            "example.com"
        );
    }

    #[test]
    fn test_host_string_ipv4() {
        use url::Host;
        assert_eq!(
            ReplicaURL::host_string(Host::Ipv4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            "127.0.0.1"
        );
    }

    #[test]
    fn test_host_string_ipv6() {
        use url::Host;
        assert_eq!(
            ReplicaURL::host_string(Host::Ipv6(std::net::Ipv6Addr::LOCALHOST)),
            "::1"
        );
    }
}
