use std::env;

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub connection_string: String,
}

impl DatabaseConfig {
    /// 从环境变量加载数据库配置
    ///
    /// 环境变量:
    /// - DATABASE_URL: 完整的数据库连接字符串 (优先级最高)
    /// - DB_TYPE: 数据库类型 (postgres, mysql)
    /// - DB_HOST: 数据库主机
    /// - DB_PORT: 数据库端口
    /// - DB_NAME: 数据库名称
    /// - DB_USER: 数据库用户
    /// - DB_PASSWORD: 数据库密码
    ///
    /// 示例:
    /// - PostgreSQL: DATABASE_URL=postgres://user:pass@localhost:5432/mydb
    /// - MySQL: DATABASE_URL=mysql://user:pass@localhost:3306/mydb
    pub fn from_env() -> Self {
        // 优先使用 DATABASE_URL
        if let Ok(url) = env::var("DATABASE_URL") {
            return DatabaseConfig {
                db_type: Self::detect_db_type(&url),
                connection_string: url,
            };
        }

        // 否则从各个环境变量组装
        let db_type = env::var("DB_TYPE").unwrap_or_else(|_| "postgres".to_string());

        let connection_string = match db_type.as_str() {
            "postgres" | "postgresql" => {
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "rustset".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
                let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "".to_string());
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    user, password, host, port, name
                )
            }
            "mysql" | "mariadb" => {
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "rustset".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
                let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "".to_string());
                format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, name)
            }
            _ => {
                // 默认 PostgreSQL
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "rustset".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
                let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "".to_string());
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    user, password, host, port, name
                )
            }
        };

        DatabaseConfig {
            db_type: Self::detect_db_type(&connection_string),
            connection_string,
        }
    }

    /// 从连接字符串检测数据库类型
    fn detect_db_type(conn_str: &str) -> String {
        let lower = conn_str.to_lowercase();
        if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
            "postgresql".to_string()
        } else if lower.starts_with("mysql://") || lower.starts_with("mariadb://") {
            "mysql".to_string()
        } else {
            "postgresql".to_string()
        }
    }

    /// 创建默认配置 (PostgreSQL)
    pub fn default_postgres() -> Self {
        DatabaseConfig {
            db_type: "postgresql".to_string(),
            connection_string: "postgres://postgres:postgres@localhost:5432/rustset".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_db_type() {
        assert_eq!(
            DatabaseConfig::detect_db_type("postgres://localhost/mydb"),
            "postgresql"
        );
        assert_eq!(
            DatabaseConfig::detect_db_type("mysql://localhost/mydb"),
            "mysql"
        );
    }

    #[test]
    fn test_default_postgres() {
        let config = DatabaseConfig::default_postgres();
        assert_eq!(config.db_type, "postgresql");
        assert!(config.connection_string.starts_with("postgres://"));
    }
}
