//! Scan engine manager
//!
//! Manages scanning operations and provides a unified interface for different scan engines.

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::scanners::{rustscan::RustScan, service_detector::ServiceDetector};
use shared::{
    AdvancedScanConfig, AdvancedScanTask, IPScanResult, PortDetail, PortInfo, QuickScanResult,
    ScanStrategy, ServiceFingerprint, TaskStatus,
};

/// Active scan job
#[derive(Clone)]
pub struct ScanJob {
    pub id: String,
    pub status: TaskStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub progress: f32,
}

/// Scan manager - handles all scanning operations
pub struct ScanManager {
    rustscan: RustScan,
    service_detector: ServiceDetector,
    active_jobs: Arc<TokioMutex<Vec<ScanJob>>>,
}

impl ScanManager {
    /// Create a new scan manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            rustscan: RustScan::new(),
            service_detector: ServiceDetector::new(),
            active_jobs: Arc::new(TokioMutex::new(Vec::new())),
        })
    }

    /// Quick port scan for a single target
    pub async fn quick_scan(&self, target: &str, ports: &[u16]) -> QuickScanResult {
        let result = self.rustscan.scan_target(target, ports).await;

        let port_infos: Vec<PortInfo> = result
            .open_ports
            .iter()
            .map(|&p| PortInfo {
                port: p,
                is_open: true,
                service: None,
                version: None,
                banner: None,
                is_bound: false,
                system_name: None,
                middleware: None,
                created_by: None,
                updated_by: None,
            })
            .collect();

        QuickScanResult {
            ip: result.ip,
            is_alive: result.is_alive,
            open_ports: port_infos,
            fingerprint: None,
            scanned_at: Utc::now(),
        }
    }

    /// Detailed scan with service detection
    pub async fn detailed_scan(&self, target: &str, ports: &[u16]) -> IPScanResult {
        let start = std::time::Instant::now();
        let scan_result = self.rustscan.scan_target(target, ports).await;

        // Detect services on open ports
        let open_ports: Vec<PortDetail> = if scan_result.is_alive {
            let service_map = self
                .service_detector
                .detect_services(target, &scan_result.open_ports)
                .await;

            scan_result
                .open_ports
                .into_iter()
                .map(|port| {
                    let service_info = service_map.get(&port);

                    PortDetail {
                        id: Uuid::new_v4().to_string(),
                        asset_id: target.to_string(),
                        ip: target.to_string(),
                        port,
                        protocol: "TCP".to_string(),
                        status: "open".to_string(),
                        service_name: service_info.map(|s| s.name.clone()),
                        service_version: service_info.and_then(|s| s.version.clone()),
                        service_banner: service_info.and_then(|s| s.banner.clone()),
                        system_name: None,
                        system_type: None,
                        system_url: None,
                        middleware: None,
                        framework: None,
                        language: None,
                        environment: None,
                        department: None,
                        owner: None,
                        owner_contact: None,
                        vulnerability_level: None,
                        has_vulnerability: false,
                        vulnerability_count: 0,
                        fingerprint: service_info.map(|s| ServiceFingerprint {
                            service: Some(s.name.clone()),
                            product: None,
                            version: s.version.clone(),
                            extra_info: s.banner.clone(),
                            cpe: None,
                            os_match: None,
                            confidence: s.confidence,
                        }),
                        is_bound: false,
                        notes: None,
                        last_scanned: Some(Utc::now()),
                        created_at: Some(Utc::now()),
                        updated_at: Some(Utc::now()),
                        created_by: None,
                        updated_by: None,
                    }
                })
                .collect()
        } else {
            vec![]
        };

        IPScanResult {
            id: Uuid::new_v4().to_string(),
            ip_zone_id: "default".to_string(),
            ip: target.to_string(),
            is_alive: scan_result.is_alive,
            hostname: None,
            mac_address: None,
            open_ports,
            os_fingerprint: None,
            device_type: None,
            confidence: 80,
            scan_time: Utc::now(),
            scan_duration_ms: start.elapsed().as_millis() as i64,
            scanned_by: "system".to_string(),
        }
    }

    /// Scan multiple targets
    pub async fn scan_multiple(&self, targets: &[String], ports: &[u16]) -> Vec<QuickScanResult> {
        let results = self.rustscan.scan_targets(targets, ports).await;

        results
            .into_iter()
            .map(|result| QuickScanResult {
                ip: result.ip.clone(),
                is_alive: result.is_alive,
                open_ports: result
                    .open_ports
                    .iter()
                    .map(|&p| PortInfo {
                        port: p,
                        is_open: true,
                        service: None,
                        version: None,
                        banner: None,
                        is_bound: false,
                        system_name: None,
                        middleware: None,
                        created_by: None,
                        updated_by: None,
                    })
                    .collect(),
                fingerprint: None,
                scanned_at: Utc::now(),
            })
            .collect()
    }

    /// Execute an advanced scan task
    pub async fn execute_advanced_scan(
        &self,
        task_id: String,
        targets: Vec<String>,
        config: AdvancedScanConfig,
    ) -> Result<AdvancedScanTask, Box<dyn std::error::Error>> {
        let total_count = targets.len() as u32;
        let mut results = Vec::new();
        let mut scanned_count = 0;

        // Determine ports based on strategy
        let ports = match config.strategy {
            ScanStrategy::Quick => RustScan::get_common_ports("TOP100"),
            ScanStrategy::Standard => RustScan::get_common_ports("TOP1000"),
            ScanStrategy::Full => RustScan::get_common_ports("ALL"),
            ScanStrategy::Custom(ref custom) => custom.clone(),
            ScanStrategy::Cloud => RustScan::get_common_ports("TOP100"),
        };

        // Track active job
        let job_start_time = Utc::now();
        let job = ScanJob {
            id: task_id.clone(),
            status: TaskStatus::Running,
            start_time: job_start_time,
            progress: 0.0,
        };
        self.active_jobs.lock().await.push(job.clone());

        // Scan each target
        for target in targets.iter() {
            let scan_result = if config.service_detection {
                let detailed = self.detailed_scan(target, &ports).await;
                QuickScanResult {
                    ip: detailed.ip.clone(),
                    is_alive: detailed.is_alive,
                    open_ports: detailed
                        .open_ports
                        .iter()
                        .map(|p| PortInfo {
                            port: p.port,
                            is_open: p.status == "open",
                            service: p.service_name.clone(),
                            version: p.service_version.clone(),
                            banner: p.service_banner.clone(),
                            is_bound: p.is_bound,
                            system_name: p.system_name.clone(),
                            middleware: p.middleware.clone(),
                            created_by: None,
                            updated_by: None,
                        })
                        .collect(),
                    fingerprint: detailed.os_fingerprint,
                    scanned_at: detailed.scan_time,
                }
            } else {
                self.quick_scan(target, &ports).await
            };

            results.push(scan_result);
            scanned_count += 1;

            // Update progress
            let progress = scanned_count as f32 / total_count as f32;
            self.update_job_progress(&task_id, progress).await;
        }

        // Complete job
        self.complete_job(&task_id).await;

        Ok(AdvancedScanTask {
            id: task_id,
            name: format!("Scan of {} targets", total_count),
            targets,
            config,
            status: TaskStatus::Completed,
            progress: 1.0,
            current_target: None,
            scanned_count,
            total_count,
            start_time: Some(job.start_time),
            end_time: Some(Utc::now()),
            results,
            cloud_mappings: vec![],
            error_message: None,
            created_by: None,
        })
    }

    /// Update job progress
    async fn update_job_progress(&self, job_id: &str, progress: f32) {
        let mut jobs = self.active_jobs.lock().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.progress = progress;
        }
    }

    /// Mark job as completed
    async fn complete_job(&self, job_id: &str) {
        let mut jobs = self.active_jobs.lock().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = TaskStatus::Completed;
            job.progress = 1.0;
        }
    }

    /// Get all active jobs
    pub async fn get_active_jobs(&self) -> Vec<ScanJob> {
        self.active_jobs.lock().await.clone()
    }
}
