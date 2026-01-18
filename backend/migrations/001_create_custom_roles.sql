-- 创建自定义角色表
CREATE TABLE IF NOT EXISTS custom_roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_custom_roles_name ON custom_roles(name);

-- 插入默认角色
INSERT INTO custom_roles (name, description, permissions) VALUES
('系统管理员', '拥有所有权限的系统管理员', '{"can_access_general":true,"can_view_dashboard":true}'),
('安全管理员', '负责资产、扫描和风险管理的安全管理员', '{"can_access_general":true}'),
('审计员', '只能查看日志的审计员', '{"can_access_audit":true,"can_view_audit_logs":true}')
ON CONFLICT (name) DO NOTHING;
