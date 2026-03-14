.PHONY: all build run test clean docker docker-up docker-down migrate

# 默认目标
all: build

# 构建项目
build:
	cargo build --release

# 运行后端
run:
	cd backend && cargo run

# 运行测试
test:
	cargo test --all

# 清理构建
clean:
	cargo clean
	rm -rf target/

# Docker 构建
docker:
	docker build -t rustset:latest .

# Docker 启动
docker-up:
	docker-compose up -d

# Docker 停止
docker-down:
	docker-compose down

# 数据库迁移
migrate:
	cd backend && cargo run -- --migrate

# 开发模式
dev:
	cargo watch -x "run -p backend"

# 前端开发
frontend:
	cd frontend && dx serve --port 8080

# 格式化代码
fmt:
	cargo fmt --all
	cargo clippy --fix --allow-dirty

# 检查代码
check:
	cargo clippy --all-targets --all-features -- -D warnings
