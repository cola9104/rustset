# 上游差异清单（本地 apps/web vs kairos-platform @ 0d9c90ef）

基线：上游 [kairos-platform](https://github.com/cola9104/kairos-platform) 固定提交
`0d9c90efef5c178703da35294acd381ef12cd0cf`，本地克隆 `~/upstream/kairos-platform`。
对比命令：`diff -rq upstream/kairos-platform/apps/web rustset/apps/web`（排除 node_modules/dist/锁文件）。
后续按模块审查更新上游时，以本清单为起点。

## 本地刻意保留的差异

| 文件 | 差异 | 理由 |
| --- | --- | --- |
| `apps/web-antd/.env` | 标题 RustSet 管理平台（上游：启元智能中台）；命名空间 `rustset-vben-antd-v2` | 品牌与缓存版本（v2 使旧 Dioxus 时代的本地存储失效） |
| `apps/web-antd/.env.development` | 默认密码 admin123（上游 Admin#123456） | 与本地 yudao 基线种子账号一致，P2 bootstrap 落地时统一 |
| `apps/web-antd/src/preferences.ts` | logo 指向 `/static/rustset-logo.svg` | 品牌资源（几何图标复用，无文字） |
| `packages/@core/ui-kit/shadcn-ui/.../back-top.vue` | bottom: 20（上游 80） | 本地 UI 微调 |
| `packages/effects/plugins/src/echarts/echarts.ts` | 不注册 GraphChart | 仅供上游扫描仪表盘使用；P4 若引入 dashboard 页面需回补 |
| `apps/web-antd/src/api/ai/model/model/index.ts` 及 model 视图 | discoverModels 参数为必填（上游可选） | 与本地后端 handler 严格校验一致；P4 逐页核销时复核 |
| `apps/web-antd/package.json` | 无 @xterm 依赖 | 本地不引入上游终端类页面（Pent/Scan） |

## 工具链差异（bun 迁移，提交 0fb7dca）

- `package.json`：bun 原生 workspace 布局（catalog/overrides/patchedDependencies 并入 package.json，删除 pnpm-workspace.yaml）；`packageManager: bun@1.4.2`；trustedDependencies；preinstall 守卫移除；pnpm 脚本调用全部改为 turbo/bun 直调。
- `pnpm-workspace.yaml`：已删除（playground 死引用随之消失；上游该清单同样引用了不存在的 playground 目录，属上游冗余）。
- `pnpm-lock.yaml` 删除，`bun.lock` 入库；turbo ^2.11.3（支持 bun lockfile v2）；turbo.json 增加 stub 任务。
- `lefthook.yml`、`scripts/clean.mjs`、`internal/node-utils/scripts/build.mjs`：bunx/bun 适配。

## 已引入的上游修正（提交于 kairos-web-shell）

- `packages/utils/src/helpers/generate-menus.ts` + `__tests__/generate-menus.test.ts`：无 componentName 不启用 KeepAlive；菜单分组默认重定向到第一个可见子页面。
- `apps/web-antd/src/plugins/form-create/index.ts`：全量注册 ant-design-vue，保证页面直接使用 a-* 标签可解析。

## 未引入的上游内容（按方案排除，二期再议）

- Pent 业务：`api/pent`、`views/pent`、`router/routes/modules/pent.ts`。
- Scan 执行体系：`router/routes/modules/scan.ts`、`api/scan/{cloud-service-asset,dashboard,index,infrastructure-device,port-detail,risk-verification,vulnerability-rule}.ts`、`views/asset-ops/{cabinet,cloud-service-asset,components,dashboard,network-device,physical-server,scanner,storage-device}`。
- AI 扩展：`api/ai/{mindmap,workflow}`、`api/ai/model/apiKey`、`views/ai/{mindmap,workflow}`、`views/ai/model/apiKey`（本地基线菜单已含相应权限码，P4 按批次核销）。
- `utils/client-request-id.ts` + 测试（本地无引用）。
- `public/static/kairos-platform-logo.svg`。
