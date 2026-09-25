# P0 基线与清单报告

> 历史归档：本文保存迁移启动时的现场快照，其中的旧产品名、目录和功能状态仅用于追溯，不代表当前 RustSet 实现。

执行日期：2026-09-24。执行人：AI agent（ZCode）。对应方案：`docs/kairos-migration-plan.md` 第 7 节 P0 阶段。

结论：P0 完成。基线已备份、上游已固定、基线检查全部执行、清单齐备。计划第 10 节的 P0 待核实项全部有了结论（见 §7）。P1 可以启动。

## 1. 基线保存

| 项                 | 值                                                                                                                    |
| ------------------ | --------------------------------------------------------------------------------------------------------------------- |
| HEAD               | `54247c1cc1d4f58920f094a7ac4f1df0ed8e7fbb`                                                                            |
| 完整备份           | `/home/xiuran/rustset-baselines/rustset-workspace-20260924-142712.tar.gz`（85M，3356 文件，含 .git 与全部未跟踪文件） |
| 辅助清单           | 同目录 `baseline-20260924-142712-{HEAD,git-status,workspace.diff,untracked}.txt`                                      |
| cargo 测试全量日志 | `/home/xiuran/rustset-baselines/p0-cargo-test-full.log`                                                               |

**工作区变更性质（重要）**：git 显示 161 个文件"修改"、约 39.7 万行增删，但 `git diff HEAD --ignore-cr-at-eol` 为**空**——全部是 LF→CRLF 行尾改写，内容与 HEAD 完全一致（无 .gitattributes、autocrlf 未设置，应为工具批量改写所致）。真正的实质内容只有 31 个未跟踪文件：

- `apps/web/apps/web-antd/src/views/asset-ops/`（Kairos 资产页面，未纳入版本控制）
- `apps/web/apps/web-antd/src/api/scan/`（Kairos 扫描 API 封装）
- `docs/kairos-migration-plan.md`（本方案）

CRLF 改写已造成实际破坏：shell 脚本无法执行（`set -euo pipefail\r` 报错）。P0 仅将 `script/test-database-migrations.sh` 转回 LF。**P1 建议**：添加 `.gitattributes`（`* text=auto eol=lf`）并归一化全仓行尾（有备份兜底），否则后续所有 diff 都会被 39 万行噪音淹没。

## 2. 上游固定

- 上游：`cola9104/kairos-platform`，固定提交 `0d9c90efef5c178703da35294acd381ef12cd0cf`。
- 本地克隆：`/home/xiuran/upstream/kairos-platform`（blob:none 过滤克隆，已 checkout 到固定提交）。
- 与本仓库 `apps/web` 的目录级对比：上游同样**没有** `playground` 目录（其 workspace 清单冗余引用见 §6）。

## 3. 数据库核实（计划待核实项：数据库/租户）

本地唯一真实库为 docker 卷 `docker_rust-toon-postgres` 中的 `rust_toon` 库（旧容器 `rust-toon-postgres`）：

- **无真实业务数据**。内容为 yudao 基线演示数据：3 租户（id=1 芋道源码、121 小租户、122 测试租户）、1 用户（admin）、1 角色（super_admin）、288 条菜单。
- **迁移历史脱节**：`_sqlx_migrations` 有 22 条旧链记录（Toonflow 时代），与仓库链（0001/0002）版本和校验和均不匹配。
- **网关实测**：对旧库启动 `cargo run -p rustset-gateway` 立即失败：`Error: migration 3 was previously applied but is missing in the resolved migrations`。SQLx 拒绝接管，符合 AGENTS.md"迁移链重置后旧库需重建一次"的预期。
- **结论**：旧库按可弃处理，本地部署直接用空库 + 新 compose 容器（`rustset-postgres`，密码 rustset）。无数据回填负担。资产多租户问题（原待核实项）随之消解：资产模块从未有数据，按方案默认"单租户 + 服务端约束业务租户"执行。
- 旧库 48 张表里没有 `infra_asset` 等资产表——`0002_asset_management.sql` 从未在任何环境真实应用过（仅空库测试验证过可执行）。

## 4. 基线检查结果

### 4.1 Rust 后端

- `cargo test --workspace`：**36 套件、41 通过、0 失败**（含 dioxus 前端 crate 编译；2 个套件有 `#[ignore]` 的迁移测试，由脚本执行）。

### 4.2 数据库迁移（`script/test-database-migrations.sh`，一次性 postgres:18 容器）

发现并处置了 3 个问题：

1. **脚本 crate 名过时**：引用 `rust-toon-framework-database`，实际 crate 是 `rustset-framework-database`，脚本无法运行。已修复（P0 最小必要修改）。
2. **脚本自身被 CRLF 改写**无法执行。已转回 LF。
3. **迁移数量断言过时**（计划 2.3.2 证实）：空库实际应用 2 个迁移，测试断言 `applied == 1`。已修正为 2。

修正后测试推进到第 164 行暴露**现存真实失败**：`0001_initial.sql` 基线给 `system_notify_message` 播种了 10 行演示数据，与"运行时表必须初始为空"的断言冲突。处置决策留给 P3：要么从 0001 剥离演示通知数据（更符合测试意图），要么调整断言。**这是当前唯一未解决的测试失败**。

### 4.3 前端（bun，用户指定工具链）

用户环境：bun 1.4.2；**全局 pnpm 已损坏**（任何目录 `pnpm --version` 报 "v11.13.0 is a broken release"，pnpm-managed 自更新装坏），node 24 经 fnm 可用。结论：**bun 是唯一可行工具链，P1 应正式切换**。

bun 链路实测全绿：

| 步骤       | 命令                                                  | 结果                                                                                   |
| ---------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------- |
| 安装       | `bun install --ignore-scripts`                        | ✓ 2200 包（`--ignore-scripts` 用于绕过 Vben 的 `npx only-allow pnpm` preinstall 守卫） |
| 内部包构建 | `bunx tsdown` / `bunx tsc`（node-utils、vite-config） | ✓（原生脚本写死 `pnpm exec`，需 bunx 替代）                                            |
| 类型检查   | `bun run typecheck`（web-antd，vue-tsc）              | ✓ 0 错误                                                                               |
| 生产构建   | `bunx vite build --mode production`（web-antd）       | ✓ 8.16s，dist + dist.zip 生成                                                          |

**P1 需要的工具链改造清单**（否则 bun 每次都要手工绕行）：

- `pnpm-workspace.yaml`：移除指向不存在目录的 `playground`（**已在本轮完成**，上游同样没有该目录，属上游冗余而非本地缺失；已记入上游差异）。bun 硬性要求 workspace 目录存在，pnpm 只是容忍。
- `package.json` preinstall 守卫 `only-allow pnpm` 需允许 bun。
- 各内部包 stub/build 脚本的 `pnpm exec` → 直接调用或 bunx。
- `packageManager` 字段与 `engines` 更新为 bun；决定 bun.lock 入库、pnpm-lock.yaml 退役（`bun install` 会自动从 pnpm-lock 迁移生成 bun.lock，本轮已生成）。
- 注意：bun install 曾把 pnpm-workspace.yaml 内容"移动"进 package.json（已用 `git checkout` 恢复），切 bun 后此迁移要显式决策。

## 5. 功能映射汇总（计划 §5.2 的全量基线）

### 5.1 前端路由（Dioxus，`apps/web-dioxus/src/main.rs` Route 枚举）

共 84 条 + `/:..route` 通配：auth 3；dashboard 2；system 28；infra 21（含构建/Rust/PG/traces/swagger 等开发者页面）；asset 12；asset 别名 9（/cloud /provider /room 等）；ai 10。逐条映射沿用计划 §5.2 表格，P4 按批次核销。

### 5.2 后端 API（gateway 装配，共 485 条路由 + 健康检查/OpenAPI/审计层）

| 模块          | 路由数 | CurrentUser 引用          |
| ------------- | ------ | ------------------------- |
| system-server | 180    | 87 处 / 18 文件中 11 个   |
| infra-server  | 223    | **0 处 / 13 文件中 0 个** |
| ai-server     | 79     | 89 处                     |
| media-server  | 3      | 9 处                      |

### 5.3 数据库表（迁移文件声明）

- `0001_initial.sql`（11MB 合并基线）：public 48 表（system*\* 37、infra*\* 10、yudao_demo 3）+ 分 schema：ai 13、toonflow 26、toon 3、media 1。
- `0002_asset_management.sql`：15 张 `infra_*` 资产表（provider/room/zone/cloud/security/asset/business/ticket/task/risk）。

### 5.4 权限码基线（0001 菜单数据，共 281 个 `x:y:z` 权限码）

| 域     | 数量 | 说明                             |
| ------ | ---- | -------------------------------- |
| system | 86   | 保留                             |
| ai     | 62   | 保留                             |
| bpm    | 0    | 已由 0016 移除，不纳入本项目     |
| infra  | 42   | 基础设施，保留                   |
| bid    | 25   | yudao 招投标遗留，后端无对应模块 |
| toon   | 18   | Toonflow 遗留                    |

bpm 权限码已由 0016 清理；其余遗留权限继续按实际后端能力收敛。

## 6. 已证实的风险与计划 2.3 对照

| 计划 2.3 条目               | P0 核实结果                                                                                                                                                           |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. 业务路由鉴权缺口         | **证实且更严重**：infra-server 全部 223 条路由零认证提取；全局中间件无 Authorization 头即放行（`database_auth.rs:36-38`）。修复归 P2                                  |
| 2. 迁移数量断言过时         | 证实，已修（1→2）                                                                                                                                                     |
| 3. 历史迁移自动认领         | `reconcile_migrations` 对已有 `_sqlx_migrations` 的库直接交给 SQLx 校验；对旧库实测报 VersionMissing。快照恢复路径（无历史表）仍会全部认领，P3 需显式 schema 校验接管 |
| 4. bootstrap 文档与代码不符 | 证实：`bootstrap.rs` 仅验证超管存在；`BOOTSTRAP_ADMIN_*` 在 Rust 代码**零引用**，AGENTS.md 描述的创建机制不存在。修复归 P2                                            |
| 5. 租户/数据范围            | `framework/tenant` 仍为空 crate（仅目录，无 src 实现文件）。P2 落地                                                                                                   |

## 7. 计划 §10 待核实项结论

| 待核实项                      | 结论                                                                                                                          |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| 是否完整导入 Kairos Pent/Scan | 维持默认：仅前端、权限及现有业务页面（未收到变更指示）                                                                        |
| 数据库是否有真实数据          | **无**。yudao 演示数据，旧库可弃，空库重建                                                                                    |
| 资产是否多租户使用            | **从未使用**（无表无数据）。按单租户 + 服务端约束执行                                                                         |
| 必须保留的完整功能清单        | 以 §5 映射为准；Toonflow 媒体业务在 0001 基线和 media/ai 模块中存在，属"现有业务"，是否保留待用户在 P1 前确认（方案默认保留） |
| 上游新增资产字段/页面是否纳入 | 未纳入决策，P4 按批次单列                                                                                                     |
| 品牌与产品名称                | RustSet（未变）                                                                                                               |
| 工具链                        | **新增决策：bun 取代 node/pnpm**（pnpm 已损坏，bun 全链路验证通过），P1 落地改造                                              |

## 8. P0 期间对工作区的修改（全部有备份）

| 文件                                                   | 修改                                          | 性质                                          |
| ------------------------------------------------------ | --------------------------------------------- | --------------------------------------------- |
| `script/test-database-migrations.sh`                   | crate 名 `rust-toon-` → `rustset-`；CRLF → LF | 必要修复                                      |
| `crates/framework/database/tests/migrations.rs`        | 迁移数断言 1 → 2                              | 必要修复（内容与真实链一致）                  |
| `apps/web/pnpm-workspace.yaml`                         | 移除 `playground`                             | bun 兼容，记入上游差异                        |
| `apps/web/bun.lock`（新增，未跟踪）                    | bun 安装锁定文件                              | 待 P1 决定入库                                |
| `apps/web/node_modules`、`internal/*/dist`、`dist.zip` | 构建产物                                      | 可再生，gitignore 覆盖（bun.lock 需补充决策） |
| `docs/migration/p0-baseline.md`                        | 本报告                                        | 交付物                                        |

## 8.1 遗留问题（移交后续阶段）

1. `system_notify_message` 10 行演示数据 vs 运行时空表断言——**唯一未解决测试失败**，P3 决策。
2. 全仓 CRLF 归一化 + `.gitattributes`——P1。
3. bun 工具链正式化（§4.3 清单）——P1。
4. infra 鉴权全覆盖——P2。
5. bootstrap 管理员创建机制实现或文档纠正——P2。
6. `sql/postgresql/0001_initial.sql.bak`（2MB）来历不明、无引用——建议 P1 清理前确认。

## 9. 环境当前状态（收尾时）

- 旧容器 `rust-toon-postgres` 已启动（仅为检查数据，可随时停）；新 compose 栈未创建。
- 网关未运行。前端 `bun.lock` 已生成，依赖已安装。
