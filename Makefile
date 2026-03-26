.PHONY: all build start stop restart status dev run backend frontend test clean docker docker-up docker-down migrate fmt check

# 默认目标
all: build

# 启动前后端
start:
	./start.sh start

# 停止前后端
stop:
	./stop.sh

# 重启前后端
restart:
	./start.sh restart

# 查看状态
status:
	./start.sh status

# 构建项目
build:
	cargo build --release -p backend
	cd frontend && dx build --platform web --release

# 开发模式
dev: start

# 兼容旧入口
run:
	./start.sh start

# 仅运行后端
backend:
	cargo run -p backend

# 仅运行前端
frontend:
	cd frontend && VITE_API_BASE=http://127.0.0.1:3003/api dx serve --port 8080

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

# 格式化代码
fmt:
	cargo fmt --all

# 检查代码
check:
	cargo clippy --all-targets --all-features -- -D warnings
