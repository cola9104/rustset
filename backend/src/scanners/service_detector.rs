//! Service detection module
//!
//! Provides service fingerprinting capabilities for common services.

use std::collections::HashMap;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use regex::Regex;

/// Detected service information
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub version: Option<String>,
    pub banner: Option<String>,
    pub confidence: i32,
}

/// Service detector for identifying services on open ports
pub struct ServiceDetector {
    probe_timeout: Duration,
    service_patterns: HashMap<u16, Vec<ServiceProbe>>,
}

impl ServiceDetector {
    /// Create a new service detector
    pub fn new() -> Self {
        let mut detector = Self {
            probe_timeout: Duration::from_millis(2000),
            service_patterns: HashMap::new(),
        };
        detector.load_service_patterns();
        detector
    }

    /// Set probe timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.probe_timeout = timeout;
        self
    }

    /// Load service detection patterns
    fn load_service_patterns(&mut self) {
        // HTTP/HTTPS services
        self.service_patterns.insert(80, vec![
            ServiceProbe::new("http", "GET / HTTP/1.0\r\n\r\n", vec![
                (r"Server:\s*(.+)", "version"),
                (r"nginx", "name"),
                (r"Apache", "name"),
                (r"IIS", "name"),
            ]),
        ]);

        self.service_patterns.insert(443, vec![
            ServiceProbe::new("https", "GET / HTTP/1.0\r\n\r\n", vec![
                (r"Server:\s*(.+)", "version"),
            ]),
        ]);

        self.service_patterns.insert(8080, vec![
            ServiceProbe::new("http-alt", "GET / HTTP/1.0\r\n\r\n", vec![
                (r"Server:\s*(.+)", "version"),
            ]),
        ]);

        // Additional HTTP ports (common alternate ports)
        for port in [8000, 8001, 8008, 8888, 9000, 9090, 3000, 5000, 5001, 8443] {
            self.service_patterns.insert(port, vec![
                ServiceProbe::new("http", "GET / HTTP/1.0\r\n\r\n", vec![
                    (r"Server:\s*(.+)", "version"),
                    (r"HTTP/\d\.\d\s+200", "banner"),
                ]),
            ]);
        }

        // SSH
        self.service_patterns.insert(22, vec![
            ServiceProbe::new("ssh", "", vec![
                (r"SSH-[\d.]+-(.+)", "version"),
                (r"OpenSSH", "name"),
                (r"Dropbear", "name"),
            ]),
        ]);

        // FTP
        self.service_patterns.insert(21, vec![
            ServiceProbe::new("ftp", "", vec![
                (r"220\s+(.+)", "banner"),
                (r"vsftpd", "name"),
                (r"ProFTPD", "name"),
                (r"FileZilla", "name"),
                (r"Microsoft FTP", "name"),
            ]),
        ]);

        // SMTP
        self.service_patterns.insert(25, vec![
            ServiceProbe::new("smtp", "", vec![
                (r"220\s+(.+)", "banner"),
                (r"Postfix", "name"),
                (r"Sendmail", "name"),
                (r"Exim", "name"),
                (r"Microsoft ESMTP", "name"),
            ]),
        ]);

        self.service_patterns.insert(587, vec![
            ServiceProbe::new("smtp-submission", "", vec![
                (r"220\s+(.+)", "banner"),
            ]),
        ]);

        // DNS
        self.service_patterns.insert(53, vec![
            ServiceProbe::new("dns", "\x00\x00\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00", vec![]),
        ]);

        // POP3
        self.service_patterns.insert(110, vec![
            ServiceProbe::new("pop3", "", vec![
                (r"\+OK\s+(.+)", "banner"),
                (r"Dovecot", "name"),
                (r"Courier", "name"),
            ]),
        ]);

        // IMAP
        self.service_patterns.insert(143, vec![
            ServiceProbe::new("imap", "", vec![
                (r"\*\s+OK\s+(.+)", "banner"),
                (r"Dovecot", "name"),
                (r"Courier", "name"),
                (r"Microsoft Exchange", "name"),
            ]),
        ]);

        // MySQL
        self.service_patterns.insert(3306, vec![
            ServiceProbe::new("mysql", "", vec![
                (r"(\d+\.\d+\.\d+)", "version"),
                (r"MariaDB", "name"),
                (r"MySQL", "name"),
            ]),
        ]);

        // PostgreSQL
        self.service_patterns.insert(5432, vec![
            ServiceProbe::new("postgresql", "", vec![
                (r"PostgreSQL\s+(\d+\.\d+)", "version"),
            ]),
        ]);

        // Redis
        self.service_patterns.insert(6379, vec![
            ServiceProbe::new("redis", "*1\r\n$4\r\nINFO\r\n", vec![
                (r"redis_version:(.+)", "version"),
            ]),
        ]);

        // MongoDB
        self.service_patterns.insert(27017, vec![
            ServiceProbe::new("mongodb", "", vec![
                (r"version:\s*([\d.]+)", "version"),
            ]),
        ]);

        // RDP
        self.service_patterns.insert(3389, vec![
            ServiceProbe::new("rdp", "", vec![]),
        ]);

        // VNC
        self.service_patterns.insert(5900, vec![
            ServiceProbe::new("vnc", "", vec![
                (r"RFB\s+(\d+\.\d+)", "version"),
            ]),
        ]);

        // Telnet
        self.service_patterns.insert(23, vec![
            ServiceProbe::new("telnet", "", vec![
                (r"(.+)", "banner"),
            ]),
        ]);
    }

    /// Detect service on a specific port
    pub async fn detect_service(&self, target: &str, port: u16) -> Option<ServiceInfo> {
        if let Some(probes) = self.service_patterns.get(&port) {
            for probe in probes {
                if let Some(info) = self.try_probe(target, port, probe).await {
                    return Some(info);
                }
            }
        }

        // Default service name based on port if no probes matched
        self.get_default_service(port)
    }

    /// Try a specific probe
    async fn try_probe(&self, target: &str, port: u16, probe: &ServiceProbe) -> Option<ServiceInfo> {
        let addr_str = format!("{}:{}", target, port);
        let addrs: Vec<std::net::SocketAddr> = match addr_str.to_socket_addrs() {
            Ok(a) => a.collect(),
            Err(_) => return None,
        };

        if addrs.is_empty() {
            return None;
        }

        let addr = addrs[0];

        match timeout(self.probe_timeout, TcpStream::connect(addr)).await {
            Ok(Ok(mut stream)) => {
                // Send probe if not empty
                if !probe.data.is_empty() {
                    let _ = stream.write_all(probe.data.as_bytes()).await;
                }

                // Read response
                let mut buffer = vec![0u8; 1024];
                let mut response = String::new();

                if let Ok(Ok(n)) = timeout(Duration::from_millis(500), stream.read(&mut buffer)).await {
                    response = String::from_utf8_lossy(&buffer[..n]).to_string();
                }

                // Analyze response
                self.analyze_response(&probe.service_name, &response, &probe.patterns)
            }
            _ => None,
        }
    }

    /// Analyze probe response against patterns
    fn analyze_response(&self, default_name: &str, response: &str, patterns: &[(Regex, PatternType)]) -> Option<ServiceInfo> {
        let mut name = default_name.to_string();
        let mut version = None;
        let mut banner = None;

        for (regex, field) in patterns {
            if let Some(caps) = regex.captures(response) {
                if caps.len() > 1 {
                    let value = caps.get(1).map(|m| m.as_str().to_string());

                    match *field {
                        "name" => {
                            if let Some(v) = value {
                                name = v;
                            }
                        }
                        "version" => {
                            version = value;
                        }
                        "banner" => {
                            banner = value;
                        }
                        _ => {}
                    }
                }
            }
        }

        // If we got any useful info, return it
        if !response.is_empty() || !name.is_empty() {
            let has_version = version.is_some();
            Some(ServiceInfo {
                name,
                version,
                banner: banner.or(Some(response.to_string()).filter(|s| !s.is_empty())),
                confidence: if has_version { 90 } else { 70 },
            })
        } else {
            None
        }
    }

    /// Get default service info for a port
    fn get_default_service(&self, port: u16) -> Option<ServiceInfo> {
        let service_names = vec![
            (21, "ftp"),
            (22, "ssh"),
            (23, "telnet"),
            (25, "smtp"),
            (53, "dns"),
            (80, "http"),
            (110, "pop3"),
            (143, "imap"),
            (443, "https"),
            (445, "smb"),
            (465, "smtps"),
            (587, "smtp-submission"),
            (631, "ipp"),
            (990, "ftps"),
            (993, "imaps"),
            (995, "pop3s"),
            (1433, "mssql"),
            (3306, "mysql"),
            (3389, "rdp"),
            (5432, "postgresql"),
            (5900, "vnc"),
            (6379, "redis"),
            (8080, "http-proxy"),
            (8443, "https-alt"),
            (9200, "elasticsearch"),
            (27017, "mongodb"),
        ];

        for (p, name) in service_names {
            if p == port {
                return Some(ServiceInfo {
                    name: name.to_string(),
                    version: None,
                    banner: None,
                    confidence: 50,
                });
            }
        }

        None
    }

    /// Batch detect services on multiple ports
    pub async fn detect_services(&self, target: &str, ports: &[u16]) -> HashMap<u16, ServiceInfo> {
        let mut results = HashMap::new();

        for &port in ports {
            if let Some(info) = self.detect_service(target, port).await {
                results.insert(port, info);
            }
        }

        results
    }
}

impl Default for ServiceDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Service probe configuration
struct ServiceProbe {
    service_name: String,
    data: String,
    patterns: Vec<(Regex, PatternType)>,
}

type PatternType = &'static str;

impl ServiceProbe {
    fn new(service_name: &str, data: &str, patterns: Vec<(&str, PatternType)>) -> Self {
        let compiled_patterns = patterns
            .into_iter()
            .filter_map(|(p, t)| Regex::new(p).ok().map(|r| (r, t)))
            .collect();

        Self {
            service_name: service_name.to_string(),
            data: data.to_string(),
            patterns: compiled_patterns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_detection() {
        let detector = ServiceDetector::new();

        // Test on localhost if port is open
        if let Ok(stream) = TcpStream::connect("127.0.0.1:8080").await {
            drop(stream);

            if let Some(info) = detector.detect_service("127.0.0.1", 8080).await {
                println!("Detected service: {:?}", info);
            }
        }
    }
}
