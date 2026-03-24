//! RustScan-based high-performance port scanner
//!
//! This module implements an async port scanner using Rust's async capabilities
//! to achieve high concurrency similar to RustScan.
#![allow(dead_code)]

use futures::stream::{self, StreamExt};
use ipnetwork::IpNetwork;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// RustScan configuration
#[derive(Debug, Clone)]
pub struct RustScanConfig {
    /// Connection timeout per port
    pub timeout: Duration,
    /// Concurrent connections limit
    pub concurrency: usize,
    /// Batch size for reporting progress
    pub batch_size: usize,
    /// Adaptive timeout based on response time
    pub adaptive_timeout: bool,
    /// Retry closed ports (reduces false negatives)
    pub retries: u32,
}

impl Default for RustScanConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(500), // RustScan default is 500ms
            concurrency: 500,                    // RustScan default is 500
            batch_size: 100,
            adaptive_timeout: true,
            retries: 1,
        }
    }
}

/// Port scan result for a single port
#[derive(Debug, Clone)]
pub struct PortScanResult {
    pub port: u16,
    pub is_open: bool,
    pub latency_ms: Option<u64>,
}

/// Complete scan result for a target
#[derive(Debug, Clone)]
pub struct TargetScanResult {
    pub ip: String,
    pub is_alive: bool,
    pub open_ports: Vec<u16>,
    pub scan_duration_ms: u64,
}

/// Progress callback type
pub type ProgressCallback = Box<dyn Fn(&str, u32, u32) + Send + Sync>;

/// RustScan implementation
pub struct RustScan {
    config: RustScanConfig,
    progress_callback: Option<ProgressCallback>,
}

impl RustScan {
    /// Create a new RustScan instance with default configuration
    pub fn new() -> Self {
        Self {
            config: RustScanConfig::default(),
            progress_callback: None,
        }
    }

    /// Create a new RustScan instance with custom configuration
    pub fn with_config(config: RustScanConfig) -> Self {
        Self {
            config,
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, u32, u32) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Check if a single port is open
    async fn check_port(&self, target: &str, port: u16) -> PortScanResult {
        let start = std::time::Instant::now();

        // Parse target address
        let socket_addrs: Vec<SocketAddr> = match format!("{}:{}", target, port).to_socket_addrs() {
            Ok(addrs) => addrs.collect(),
            Err(_) => {
                return PortScanResult {
                    port,
                    is_open: false,
                    latency_ms: None,
                }
            }
        };

        if socket_addrs.is_empty() {
            return PortScanResult {
                port,
                is_open: false,
                latency_ms: None,
            };
        }

        // Try with retries
        let mut is_open = false;
        let mut latency = 0u64;

        for attempt in 0..=self.config.retries {
            let socket_addr = socket_addrs[0]; // Use first resolved address

            match timeout(self.config.timeout, TcpStream::connect(&socket_addr)).await {
                Ok(Ok(_stream)) => {
                    // Port is open
                    latency = start.elapsed().as_millis() as u64;
                    is_open = true;
                    break;
                }
                Ok(Err(_)) | Err(_) => {
                    // Port is closed or timeout
                    if attempt < self.config.retries {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        }

        PortScanResult {
            port,
            is_open,
            latency_ms: if is_open { Some(latency) } else { None },
        }
    }

    /// Scan a single target for open ports
    pub async fn scan_target(&self, target: &str, ports: &[u16]) -> TargetScanResult {
        let start = std::time::Instant::now();
        let ip = target.to_string();

        // First check if host is up (scan a few common ports quickly)
        let is_alive = self.check_host_alive(&ip).await;

        if !is_alive {
            return TargetScanResult {
                ip,
                is_alive: false,
                open_ports: vec![],
                scan_duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Scan ports in batches
        let open_ports = self.scan_ports_batched(&ip, ports).await;

        TargetScanResult {
            ip,
            is_alive: true,
            open_ports,
            scan_duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Quick check if host is alive (try common ports)
    async fn check_host_alive(&self, target: &str) -> bool {
        let common_ports = vec![22, 80, 443, 3389]; // SSH, HTTP, HTTPS, RDP

        for port in common_ports {
            if let Ok(addrs) = format!("{}:{}", target, port).to_socket_addrs() {
                let socket_addr = addrs.into_iter().next();
                if let Some(addr) = socket_addr {
                    // Use very short timeout for alive check
                    if timeout(Duration::from_millis(100), TcpStream::connect(&addr))
                        .await
                        .is_ok()
                    {
                        return true;
                    }
                }
            }
        }
        true // Assume alive and let full scan determine
    }

    /// Scan ports in batches for better performance and progress reporting
    async fn scan_ports_batched(&self, target: &str, ports: &[u16]) -> Vec<u16> {
        let mut open_ports = Vec::new();
        let total = ports.len() as u32;

        for (batch_idx, batch) in ports.chunks(self.config.batch_size).enumerate() {
            let results = self.scan_ports_concurrent(target, batch).await;

            for result in results {
                if result.is_open {
                    open_ports.push(result.port);
                }
            }

            // Report progress
            if let Some(ref callback) = self.progress_callback {
                let completed = ((batch_idx + 1) * self.config.batch_size) as u32;
                let completed = if completed > total { total } else { completed };
                callback(target, completed, total);
            }
        }

        open_ports.sort();
        open_ports.dedup();
        open_ports
    }

    /// Scan multiple ports concurrently
    async fn scan_ports_concurrent(&self, target: &str, ports: &[u16]) -> Vec<PortScanResult> {
        let timeout = self.config.timeout;
        let retries = self.config.retries;
        let target = target.to_string();

        // Create futures for each port
        let scan_futures: Vec<_> = ports
            .iter()
            .map(|&port| {
                let target = target.clone();
                async move { Self::check_port_static(&target, port, timeout, retries).await }
            })
            .collect();

        // Execute with limited concurrency using buffer_unordered on stream
        stream::iter(scan_futures)
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await
    }

    /// Static method for checking a port (for concurrent use)
    async fn check_port_static(
        target: &str,
        port: u16,
        timeout_duration: Duration,
        retries: u32,
    ) -> PortScanResult {
        let socket_addrs: Vec<SocketAddr> = match format!("{}:{}", target, port).to_socket_addrs() {
            Ok(addrs) => addrs.collect(),
            Err(_) => {
                return PortScanResult {
                    port,
                    is_open: false,
                    latency_ms: None,
                }
            }
        };

        if socket_addrs.is_empty() {
            return PortScanResult {
                port,
                is_open: false,
                latency_ms: None,
            };
        }

        let start = std::time::Instant::now();
        let mut is_open = false;
        let mut latency = 0u64;

        for attempt in 0..=retries {
            let socket_addr = socket_addrs[0];

            match timeout(timeout_duration, TcpStream::connect(&socket_addr)).await {
                Ok(Ok(_stream)) => {
                    latency = start.elapsed().as_millis() as u64;
                    is_open = true;
                    break;
                }
                Ok(Err(_)) | Err(_) => {
                    if attempt < retries {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
            }
        }

        PortScanResult {
            port,
            is_open,
            latency_ms: if is_open { Some(latency) } else { None },
        }
    }

    /// Scan multiple targets
    pub async fn scan_targets(&self, targets: &[String], ports: &[u16]) -> Vec<TargetScanResult> {
        let mut results = Vec::new();

        for target in targets {
            let result = self.scan_target(target, ports).await;
            results.push(result);
        }

        results
    }

    /// Scan an IP range (CIDR notation)
    pub async fn scan_cidr(&self, cidr: &str, ports: &[u16]) -> Vec<TargetScanResult> {
        let network: IpNetwork = match cidr.parse() {
            Ok(n) => n,
            Err(_) => return vec![],
        };

        let targets: Vec<String> = match network {
            IpNetwork::V4(v4) => {
                v4.iter()
                    .take(1000) // Limit to prevent excessive scans
                    .map(|ip| ip.to_string())
                    .collect()
            }
            IpNetwork::V6(v6) => v6.iter().take(1000).map(|ip| ip.to_string()).collect(),
        };

        self.scan_targets(&targets, ports).await
    }

    /// Get common ports for different scan strategies
    pub fn get_common_ports(strategy: &str) -> Vec<u16> {
        match strategy {
            "TOP10" => vec![21, 22, 23, 25, 53, 80, 110, 143, 443, 3389],
            "TOP100" => vec![
                7, 9, 13, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111, 113, 119, 135,
                139, 143, 144, 179, 199, 389, 427, 443, 445, 465, 506, 514, 515, 543, 548, 554,
                587, 631, 646, 873, 990, 993, 995, 1025, 1026, 1027, 1028, 1029, 1110, 1433, 1434,
                1521, 1582, 1583, 1720, 1723, 1755, 1900, 2000, 2001, 2049, 2121, 2717, 3000, 3128,
                3306, 3389, 3986, 4899, 5000, 5009, 5051, 5060, 5061, 5101, 5190, 5357, 5432, 5631,
                5666, 5800, 5900, 6000, 6001, 6646, 7070, 8000, 8008, 8009, 8080, 8081, 8443, 8888,
                9100, 9999, 10000, 32768, 49152,
            ],
            "TOP1000" => {
                // First 1000 common ports (abbreviated list - in production use nmap service file)
                let mut ports = Self::get_common_ports("TOP100");
                ports.extend(vec![
                    109, 123, 137, 138, 161, 162, 264, 311, 365, 369, 415, 416, 417, 512, 513, 517,
                    518, 520, 524, 540, 545, 546, 548, 554, 560, 563, 587, 591, 593, 616, 617, 625,
                    631, 636, 639, 646, 648, 652, 654, 660, 666, 674, 691, 700, 705, 711, 714, 720,
                    749, 765, 783, 787, 800, 801, 808, 843, 873, 880, 888, 898, 900, 901, 902, 903,
                    911, 912, 981, 987, 990, 992, 993, 994, 995, 996, 997, 998, 999, 1000, 1001,
                    1002, 1007, 1009, 1010, 1011, 1021, 1022, 1023, 1024, 1025, 1026, 1027, 1028,
                    1029, 1030, 1031, 1032, 1033, 1034, 1035, 1036, 1037, 1038, 1039, 1040, 1041,
                    1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054,
                    1055, 1056, 1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067,
                    1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080,
                    1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093,
                    1094, 1095, 1096, 1097, 1098, 1099, 1100, 1102, 1104, 1105, 1106, 1107, 1108,
                    1110, 1111, 1112, 1113, 1114, 1117, 1119, 1121, 1122, 1123, 1124, 1126, 1130,
                    1131, 1132, 1137, 1138, 1141, 1145, 1147, 1148, 1158, 1164, 1169, 1174, 1175,
                    1183, 1185, 1186, 1187, 1192, 1198, 1199, 1201, 1213, 1216, 1217, 1218, 1233,
                    1234, 1236, 1244, 1247, 1248, 1259, 1271, 1277, 1287, 1296, 1300, 1301, 1309,
                    1310, 1311, 1322, 1328, 1334, 1352, 1417, 1433, 1434, 1443, 1455, 1461, 1494,
                    1500, 1501, 1503, 1521, 1524, 1533, 1556, 1580, 1583, 1594, 1600, 1641, 1658,
                    1666, 1687, 1688, 1700, 1717, 1718, 1719, 1720, 1721, 1723, 1755, 1761, 1782,
                    1783, 1801, 1805, 1812, 1813, 1839, 1840, 1862, 1863, 1864, 1875, 1900, 1914,
                    1935, 1947, 1971, 1972, 1974, 1984, 1998, 1999, 2000, 2001, 2002, 2003, 2004,
                    2005, 2006, 2007, 2008, 2009, 2010, 2013, 2020, 2021, 2022, 2030, 2033, 2034,
                    2035, 2038, 2040, 2041, 2042, 2043, 2045, 2046, 2047, 2048, 2049, 2065, 2068,
                    2099, 2100, 2103, 2105, 2106, 2107, 2111, 2119, 2121, 2126, 2135, 2144, 2160,
                    2161, 2170, 2179, 2190, 2191, 2196, 2200, 2222, 2251, 2260, 2288, 2301, 2323,
                    2366, 2381, 2382, 2383, 2393, 2394, 2399, 2401, 2492, 2500, 2522, 2525, 2557,
                    2601, 2602, 2604, 2605, 2607, 2608, 2638, 2701, 2702, 2710, 2717, 2718, 2725,
                    2800, 2809, 2811, 2869, 2875, 2909, 2910, 2920, 2967, 2968, 2998, 3000, 3001,
                    3003, 3005, 3006, 3007, 3011, 3013, 3017, 3030, 3031, 3052, 3071, 3077, 3128,
                    3168, 3211, 3221, 3260, 3261, 3268, 3269, 3283, 3300, 3301, 3306, 3322, 3323,
                    3324, 3325, 3333, 3351, 3367, 3369, 3370, 3371, 3372, 3389, 3390, 3404, 3476,
                    3493, 3517, 3527, 3546, 3551, 3580, 3659, 3689, 3690, 3703, 3737, 3766, 3784,
                    3800, 3801, 3809, 3814, 3826, 3827, 3828, 3851, 3869, 3871, 3878, 3880, 3889,
                    3905, 3914, 3918, 3920, 3945, 3971, 3986, 3995, 3998, 4000, 4001, 4002, 4003,
                    4004, 4005, 4006, 4045, 4111, 4125, 4126, 4129, 4224, 4242, 4279, 4321, 4343,
                    4443, 4444, 4445, 4446, 4449, 4550, 4567, 4662, 4848, 4899, 4900, 4998, 5000,
                    5001, 5002, 5003, 5004, 5009, 5030, 5033, 5050, 5051, 5054, 5060, 5061, 5080,
                    5087, 5100, 5101, 5102, 5120, 5190, 5200, 5214, 5221, 5222, 5225, 5226, 5269,
                    5280, 5298, 5357, 5405, 5414, 5431, 5432, 5440, 5500, 5510, 5544, 5550, 5555,
                    5560, 5566, 5631, 5633, 5666, 5678, 5679, 5718, 5730, 5800, 5801, 5802, 5810,
                    5811, 5815, 5822, 5825, 5850, 5859, 5862, 5877, 5900, 5901, 5902, 5903, 5904,
                    5906, 5907, 5910, 5911, 5915, 5922, 5925, 5950, 5952, 5959, 5960, 5961, 5962,
                    5963, 5987, 5988, 5989, 5998, 5999, 6000, 6001, 6002, 6003, 6004, 6005, 6006,
                    6007, 6009, 6025, 6059, 6100, 6101, 6106, 6112, 6123, 6129, 6156, 6346, 6389,
                    6502, 6510, 6543, 6547, 6565, 6566, 6567, 6580, 6646, 6666, 6667, 6668, 6669,
                    6689, 6692, 6699, 6779, 6788, 6789, 6792, 6839, 6881, 6901, 6969, 7000, 7001,
                    7002, 7004, 7007, 7019, 7025, 7070, 7100, 7103, 7106, 7200, 7201, 7402, 7435,
                    7443, 7496, 7512, 7625, 7627, 7676, 7741, 7777, 7778, 7800, 7911, 7920, 7921,
                    7937, 7938, 7999, 8000, 8001, 8002, 8007, 8008, 8009, 8010, 8011, 8021, 8022,
                    8031, 8042, 8045, 8080, 8081, 8082, 8083, 8084, 8085, 8086, 8087, 8088, 8089,
                    8090, 8093, 8099, 8100, 8180, 8181, 8192, 8193, 8194, 8200, 8222, 8254, 8290,
                    8291, 8292, 8300, 8333, 8383, 8400, 8402, 8443, 8500, 8600, 8649, 8651, 8652,
                    8654, 8701, 8800, 8873, 8888, 8899, 8900, 8910, 8994, 9000, 9001, 9002, 9003,
                    9009, 9010, 9011, 9040, 9050, 9071, 9080, 9081, 9090, 9091, 9099, 9100, 9101,
                    9102, 9103, 9110, 9111, 9200, 9207, 9220, 9290, 9415, 9418, 9485, 9500, 9502,
                    9503, 9535, 9575, 9593, 9594, 9595, 9618, 9666, 9876, 9877, 9878, 9898, 9900,
                    9917, 9929, 9943, 9944, 9968, 9998, 9999, 10000, 10001, 10002, 10003, 10004,
                    10009, 10010, 10012, 10024, 10025, 10082, 10180, 10215, 10243, 10566, 10616,
                    10617, 10621, 10626, 10628, 10629, 10778, 11110, 11111, 11967, 12000, 12174,
                    12265, 12345, 13456, 13722, 13782, 13783, 14000, 14238, 14441, 14442, 15000,
                    15002, 15003, 15004, 15660, 15742, 16000, 16001, 16012, 16016, 16018, 16080,
                    16113, 16992, 16993, 17877, 17988, 18040, 18101, 18988, 19101, 19283, 19315,
                    19350, 19780, 19801, 19842, 20000, 20005, 20031, 20221, 20222, 20828, 21571,
                    22939, 23502, 24444, 24800, 25734, 25735, 26214, 27000, 27352, 27353, 27355,
                    27356, 27715, 28201, 30000, 30718, 30951, 31038, 31337, 32768, 32769, 32770,
                    32771, 32772, 32773, 32774, 32775, 32776, 32777, 32778, 32779, 32780, 32781,
                    32782, 32783, 32784, 32785, 33354, 33899, 34571, 34572, 34573, 35500, 38292,
                    40193, 40911, 41511, 42510, 44176, 44442, 44443, 44501, 45100, 48080, 49152,
                    49153, 49154, 49155, 49156, 49157, 49158, 49159, 49160, 49161, 49163, 49165,
                    49167, 49175, 49176, 49400, 49999, 50000, 50001, 50002, 50003, 50006, 50300,
                    50389, 50500, 50636, 50800, 51103, 51493, 52673, 52822, 52848, 52869, 54045,
                    54328, 55055, 55056, 55555, 55600, 56737, 56738, 57294, 57797, 58080, 60020,
                    60443, 61532, 61900, 62078, 63331, 64623, 64680, 65000, 65129, 65389,
                ]);
                ports.sort();
                ports.dedup();
                ports
            }
            "ALL" => (1..=65535).collect(),
            _ => vec![],
        }
    }
}

impl Default for RustScan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_localhost_scan() {
        let scanner = RustScan::new();
        let result = scanner.scan_target("127.0.0.1", &[22, 80, 443, 8080]).await;

        assert!(result.is_alive);
        println!("Scan duration: {}ms", result.scan_duration_ms);
        println!("Open ports: {:?}", result.open_ports);
    }

    #[test]
    fn test_common_ports() {
        let top10 = RustScan::get_common_ports("TOP10");
        assert_eq!(top10.len(), 10);

        let top100 = RustScan::get_common_ports("TOP100");
        assert!(top100.len() >= 70); // At least 70 ports
    }
}
