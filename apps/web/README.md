# RustSet 管理端

这是 RustSet 的唯一正式 Web 管理端，基于 Vue 3、Vite、TypeScript、Vben 与 Ant Design Vue，使用 bun 管理依赖和执行脚本。

## 功能入口

- 资产中心：资产台账、网络策略、扫描任务、风险管理
- 配置管理：CMDB 模型、属性、实例与关系
- 基础设施中心：服务商、机房、区域、安全产品
- 云管理中心：云平台、云区域、云厂商接入
- 业务中心：业务应用、云资源、物理资源
- 运维中心：资源工单、审批规则
- AI 大模型：统一模型配置、对话、知识库、绘图、音乐与写作
- 系统管理：用户、角色、菜单、租户与租户网段等后台能力

CMDB 模型用于定义资产数据结构；AI 模型用于配置推理供应商和模型，两者是不同领域。所有 AI 功能都复用“AI 大模型 → 模型管理”的统一配置。

## 环境要求

- bun `1.4+`
- node `22.18+` 或 `24.x`（部分工具链需要）
- 已在 `127.0.0.1:8080` 启动 RustSet 网关

项目使用 bun，不使用 pnpm。

## 开发

```bash
cd apps/web
bun install
bun run dev:antd
```

开发地址为 <http://127.0.0.1:5666>。应用通过 `/api` 访问后端，开发代理会转发到 `http://127.0.0.1:8080` 并移除 `/api` 前缀。

常用配置位于：

- `apps/web-antd/.env.development`
- `apps/web-antd/.env.production`
- `apps/web-antd/src/preferences.ts`

## 检查与构建

```bash
bun run check:type
bun run build:antd
```

生产静态文件输出到 `apps/web-antd/dist`。部署时需要将 SPA 未命中路径回退到 `index.html`，并把 `/api/` 反向代理到 RustSet 网关。

## 目录约定

```text
apps/web-antd/src/
  api/                 后端接口封装
  views/asset-ops/     资产、基础设施、云、业务与运维页面
  views/cmdb/          CMDB 页面
  views/ai/            AI 页面
  views/system/        系统管理页面
packages/              复用的前端基础包
```

页面可见的产品名称统一使用 RustSet。`@vben/*` 包名属于前端基础设施的内部技术标识，不代表产品名称。
