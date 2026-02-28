//! Page routing definitions

/// Represents all available pages in the application
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Login,
    Dashboard,
    TaskCenter,
    AdvancedScanning,
    RiskCenter,
    UserManagement,
    PermissionManagement,
    AuditLogs,
    CloudProviderManagement,
    UserProfile,
    PasswordPolicyManagement,
    BusinessAcceptance,
    BusinessApplication,
    OperationsManagement,
    AutomationOrchestration,
    CloudServiceAssetManagement,
    CloudZoneManagement,
    CloudPlatformManagement,
}
