-- ============================================
-- RustSet 数据库表结构 (SQLite)
-- ============================================

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'SysAdmin', 'SecAdmin', 'Auditor', 或 'Custom'
    custom_role_id INTEGER,  -- 如果role是'Custom',关联到custom_roles表
    permissions TEXT,  -- JSON格式的权限配置
    locked_until TEXT,  -- 账户锁定截止时间
    password_changed_at TEXT,  -- 最后一次修改密码时间
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 自定义角色表
CREATE TABLE IF NOT EXISTS custom_roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    permissions TEXT NOT NULL DEFAULT '{}',  -- JSON格式的权限配置
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 资产表
CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    ip TEXT NOT NULL,
    mac TEXT,
    os TEXT,
    zone_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 资产端口表
CREATE TABLE IF NOT EXISTS asset_ports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER NOT NULL,
    port INTEGER NOT NULL,
    service_name TEXT,
    protocol TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    is_bound BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

-- 任务表
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    target_ips TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    result TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 风险表
CREATE TABLE IF NOT EXISTS risks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    asset_id INTEGER,
    asset_ip TEXT,
    asset_name TEXT,
    port INTEGER,
    risk_type TEXT,
    severity TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    details TEXT,
    resolved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE SET NULL
);

-- 网络区域表
CREATE TABLE IF NOT EXISTS zones (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cidr TEXT NOT NULL,
    priority INTEGER NOT NULL
);

-- 审计日志表
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 密码策略表
CREATE TABLE IF NOT EXISTS password_policy (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    min_length INTEGER NOT NULL DEFAULT 8,
    require_uppercase BOOLEAN NOT NULL DEFAULT 1,
    require_lowercase BOOLEAN NOT NULL DEFAULT 1,
    require_number BOOLEAN NOT NULL DEFAULT 1,
    require_special BOOLEAN NOT NULL DEFAULT 1,
    max_age_days INTEGER NOT NULL DEFAULT 90,
    min_unique_chars INTEGER NOT NULL DEFAULT 0,
    prevent_reuse INTEGER NOT NULL DEFAULT 5,
    min_strength TEXT NOT NULL DEFAULT 'medium',
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 密码历史表
CREATE TABLE IF NOT EXISTS password_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    old_password_hash TEXT NOT NULL,
    changed_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 业务资源表
CREATE TABLE IF NOT EXISTS business_resources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    applicant TEXT,
    approvers TEXT,  -- JSON array of approvers
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 云服务商配置表
CREATE TABLE IF NOT EXISTS cloud_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,  -- 'aliyun', 'tencent', 'huawei', 'aws', 'azure'
    access_key TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    region TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 云服务资产表
CREATE TABLE IF NOT EXISTS cloud_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider_id INTEGER NOT NULL,
    instance_id TEXT,
    name TEXT NOT NULL,
    instance_type TEXT,  -- 'physical' or 'virtual'
    status TEXT,
    public_ip TEXT,
    private_ip TEXT,
    cpu_cores INTEGER,
    memory_gb REAL,
    os_name TEXT,
    department TEXT,
    project TEXT,
    owner TEXT,
    region TEXT,
    created_at TEXT,
    expire_time TEXT,
    disks TEXT,  -- JSON array of disk info
    snapshots TEXT,  -- JSON array of snapshot info
    console_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    FOREIGN KEY (provider_id) REFERENCES cloud_providers(id) ON DELETE CASCADE
);

-- 高级扫描任务表
CREATE TABLE IF NOT EXISTS advanced_scan_tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    targets TEXT NOT NULL,  -- JSON array of targets
    ports TEXT,  -- JSON array of port ranges
    scan_type TEXT NOT NULL,
    engine_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    result TEXT,  -- JSON of scan results
    progress INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    terraform_state TEXT,
    cloud_asset_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 插入默认密码策略
INSERT OR IGNORE INTO password_policy (id, min_length, require_uppercase, require_lowercase, require_number, require_special, max_age_days, min_unique_chars, prevent_reuse, min_strength)
VALUES (1, 8, 1, 1, 1, 90, 0, 5, 'medium');

-- 插入默认网络区域
INSERT OR IGNORE INTO zones (id, name, cidr, priority) VALUES
('1', 'Intranet', '192.168.0.0/16', 10),
('2', 'DMZ', '10.0.0.0/8', 20);

-- 插入默认管理员用户 (密码: admin123, 需要在应用启动时生成hash)
-- 这部分在代码中处理
