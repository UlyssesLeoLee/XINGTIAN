# AIPWE — UT／IT／ST／SEC／PERF 测试设计书

文档 ID：`AIPWE-TD-001` · 版本：`v1.0.0` · 日期：2026-11-10

基线：`AIPWE-RD-001 v1.1` → `AIPWE-BD-001 v1.1` → `AIPWE-DD-001 v1.1`（包括 DADV-01..22）。

**交付状态：测试设计完成；Rust 受控 Fixture 原型代码已编写，但由于本次环境缺少 cargo/rustc，Rust 测试未实际执行。所有群组测试和详细红测状态保持“未执行”。**

## 1. 测试目标与限制

- 优先验证任务级并行、CAS、安全边界、动作效果账本与恢复，不以截图或 GUI 作为核心接口。
- 开源 Rust core 当前仅提供受控 Fixture 行为；不包含 HTML5/JS 执行、CSS、OS 沙箱、外网 Broker、持久化存储或原生 Playwright 驱动。
- Chromium Playwright 套件只是 Oracle；Fixture 内存互斥并非 Web Agent/Event Loop 正确实现；Broker 只模拟许可决策，不实际联网。
- 风险关闭规则：**文档已修正 ≠ 源码通过编译 ≠ 红测绿灯 ≠ Chrome 对标成立 ≠ 可公网部署**。

## 2. 测试层级 / 准入退出

| 测试层 | 实施对象 | 进入条件 | 退出判据 | 当前状态 |
|---|---|---|---|---|
| UT | DAG、Scheduler、CAS、Lease、Effect、Events | Crate 编译、接口契约已同步 | 全部编写的 Rust unit 测试通过、零未授权副作用 | 未执行 |
| IT | Fixture Actor、多 Agent 并发、状态漂移、幂等、Action effect | UT 通过 | 多 Agent 与效果负例有可复现实测 | 未执行 |
| ST | 真实进程沙箱、资源回收、8/16/32 ×60min | Linux sandbox 和 Native Worker 可用 | 无非预期崩溃，泄漏率按基线门槛 | 未具备执行条件 |
| SEC | 身份伪造、跨租户、SSRF、网络出口 Fencing、Prompt Injection | 可信 Broker、OS 强隔离、审计工作 | 所有 P0 红测零违规，独立安全复核 | 未具备执行条件 |
| WPT/DIFF | Web 标准、事件循环、Layout/视觉 | Native HTML/JS/Layout 具备对应 capability | 候选集、支持子集和 holdout 分母完整公开 | 未具备执行条件 |
| PERF | Native vs Chromium 配对 A/B | 安全/正确性非劣性门槛事先通过 | 16/32 一档吞吐 ≥1.20x 且资源门槛符合 DD | 未具备执行条件 |

## 3. 测试环境与数据

| 资源 | 约束 |
|---|---|
| 操作系统 | Linux x86_64 为安全与性能目标平台；Windows 11 仅开发检查 |
| Rust | cargo + rustfmt + clippy；版本由首次成功 POC 在 ADR 冻结，CI 使用受控工具链 |
| 浏览器对照 | 固定 Playwright、Chromium 版本、浏览器构建哈希；不可当作 Native 成果 |
| Fixture | `tests/fixtures/forms.html` 与固定数据，禁止访问公网 |
| 负载 | 并发 1/8/16/32/64；重点 16/32；正式稳定性 8/16/32 ×60min |
| 证据 | Commit SHA、依赖锁摘要、fixture hash、环境、trace、测试报告，敏感数据脱敏 |

## 4. 测试设计方法

采用等价类、边界值、状态迁移、组合/故障注入、权限矩阵、并行时序表、变异测试与差分 Oracle。所有高风险场景必须给出 Given / When / Then 与不可违反的系统不变量；不可仅靠 HTTP 200 或 API 202 断定业务成功。

## 5. UT 测试设计

| UT 分类 | 输入/负例 | 不变量与断言 | 样例测试位置 |
|---|---|---|---|
| DAG | 环、自环、未知节点、超界节点 | 失败 `INVALID_DAG`，不产生调度 | `graph.rs` |
| Admission | 租户混合、超额资源、队列满 | 必须拒绝超额；不得负计量 | `scheduler.rs` |
| StrictCAS | 输入值改变但 DOM 版本不变 | 旧 `form_control_revision` 导致 `StaleSnapshot` | `state.rs` / `red_regressions.rs` |
| DynamicLocator | 替换同名元素、多匹配 | 未派发前可重新定位；多匹配拒绝，不能重复派发 | `action.rs` |
| EffectLedger | 同键同摘要/异摘要、断线后重试 | 同摘要回原 ID；未知结果不得自动重放 | `effects.rs` |
| BrokerPermit | 租约撤销、目标/方法变化、重用 nonce | 未授权请求拒绝，不触网 | `security.rs` |
| SSE | seq 缺口、租户变化、保留窗截断 | `EventGap` / `Unauthorized` 显式拒绝 | `events.rs` |
| Deadline | host boot_id 改变 | 拒绝比较跨 boot 单调时间 | `time.rs` |

## 6. IT 测试设计

最小可运行受控场景：`5` 个 Agent 并发读取同一快照、`2` 个 Agent 依据同一旧版本写入；只有一项写入成功，另一项返回 `StaleSnapshot`。另外验证一次 Action 关联多个独立 Effect，2 个确认 HTTP 回执、1 个断线 `Uncertain`，不得自动发送第二次。**这不是浏览器 Web 标准完整集成测试。**

## 7. ST 测试设计

目标包括 Context 1000 次清理、强制终止失控 Worker、8/16/32 并发稳定运行 60 分钟、SSE 断线与重新订阅、整机资源 CPU 秒/PSS/RSS/cgroup 完整计量。前置依赖真实沙箱/渲染进程/持久化；目前只能以入口脚本生成负载配置，不能提供真实 ST 性能结论。

## 8. SEC 对抗式测试设计

P0 必须包含跨租户身份伪造、旧租约继续出口写、导航/重定向审批绕过、敏感跨源响应泄漏、缓存混租、恶意页面意图提权、断网后重复 POST；未构建 OS 安全隔离时，任何安全负例的 Fixture 通过都不允许宣称生产防护完成。

## 9. Playwright 对照与兼容定义

- `tests/playwright` 使用真实 Chromium，仅验证参考浏览器行为。
- Native Playwright Driver: **unsupported**；不得声称 drop-in、完整 Locator/actionability 或 CDP 完全兼容。
- 接口实现划分：Fixture API → 独立 SDK 子集 → 原生 Browser automation protocol；仅已实装且独立验证的行为可晋升 `supported`。

## 10. 性能预注册与证明

固定 A–E 混合任务、随机区组不少于五轮、独立 holdout、同硬件/权限/隔离、公开全候选测试分母。正确性、成功率、安全门槛不劣后，才计算有效任务/秒。R3 保留吞吐 ≥1.20×、单位 CPU·s 或峰值 RSS 改善 ≥15%，另一项恶化 ≤10% 的原验收门槛。无真实并行 Web 执行时禁止输出“优于 Chromium”结论。

## 11. 34 项原群组测试（编号保持不变）

| 测试 ID | 阶段 | 刺激/目标 | Oracle | 脚本对应 | 状态 |
|---|---|---|---|---|---|
| `UT-BD-001` | R0 | 固定 fixture/seed/backend/Policy；构造DAG 环检测/依赖传播正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | Cycle/Cancelled propagation | `crates/aiweb-core/src/graph.rs`（部分） | 未执行 |
| `IT-BD-002` | R0 | 固定 fixture/seed/backend/Policy；构造任务与 effects_state 正交正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 超时 POST 与已取消 Task 并存 | 待实现 | 未执行 |
| `PERF-BD-003` | R0 | 固定 fixture/seed/backend/Policy；构造资源准入与背压正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 资源超额 queue/reject，进程树无 OOM | 待实现 | 未执行 |
| `IT-BD-004` | R1 | 固定 fixture/seed/backend/Policy；构造1,000 次 Context 生命周期正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 旧租户 Key 0 泄露 | 待实现 | 未执行 |
| `DIFF-BD-005` | R1 | 固定 fixture/seed/backend/Policy；构造HTML/DOM/JS 事件顺序正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 脚本-微任务-事件时序 oracle | 待实现 | 未执行 |
| `SEC-BD-006` | R1 | 固定 fixture/seed/backend/Policy；构造OS 沙箱/跨站/SSRF正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 高风险负例 0 成功 | 待实现 | 未执行 |
| `IT-BD-007` | R2 | 固定 fixture/seed/backend/Policy；构造同页 5 读 / 2 写冲突正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 版本一致，无误提交 | `crates/aiweb-core/tests/red_regressions.rs`（部分） | 未执行 |
| `IT-BD-008` | R2 | 固定 fixture/seed/backend/Policy；构造导航与跨文档代际失效正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 旧 locator 全部 STALE_SNAPSHOT | `crates/aiweb-core/src/action.rs`（部分） | 未执行 |
| `DIFF-BD-009` | R2 | 固定 fixture/seed/backend/Policy；构造CSS/Geometry/截图一致正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 可见性和截图差分指标 | 待实现 | 未执行 |
| `SEC-BD-010` | R2 | 固定 fixture/seed/backend/Policy；构造Prompt Injection/审批正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 恶意页面不得升级权限 | 待实现 | 未执行 |
| `IT-BD-011` | R2 | 固定 fixture/seed/backend/Policy；构造Playwright fixture/API 子集正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 已支持测试 ≥98% | `tests/playwright/specs/chromium-oracle.spec.ts`（部分） | 未执行 |
| `IT-BD-012` | R2 | 固定 fixture/seed/backend/Policy；构造Playwright 服务端数据隔离正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 同账号/shared DB 无串扰 | 待实现 | 未执行 |
| `ST-BD-013` | R2 | 固定 fixture/seed/backend/Policy；构造故障局部性/uncertain effect正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 页面崩溃不影响其他 Context | 待实现 | 未执行 |
| `ST-BD-014` | R2 | 固定 fixture/seed/backend/Policy；构造8/16/32 并发 60 min正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 非预期 runtime 崩溃 0 | 待实现 | 未执行 |
| `PERF-BD-015` | R3 | 固定 fixture/seed/backend/Policy；构造A-E 同任务吞吐 A/B正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | ≥1.20x 16/32 某档 + 资源门槛 | 待实现 | 未执行 |
| `PERF-BD-016` | R3 | 固定 fixture/seed/backend/Policy；构造公平与长短混合正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | P99 wait+饥饿事件报告 | 待实现 | 未执行 |
| `PERF-BD-017` | R3 | 固定 fixture/seed/backend/Policy；构造Agent Token 成本 A/B正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | 固定模型/tokenizer/准确性 | 待实现 | 未执行 |
| `WPT-BD-018` | R1-R4 | 固定 fixture/seed/backend/Policy；构造固定 WPT + holdout 分母正常与异常分支，注入 timeout/cancel 或权限拒绝；与冻结 oracle 比较 | pass/fail/skip/unsupported 明细 | 待实现 | 未执行 |
| `RED-BD-019` | R1 | 构造同源 top window/iframe 的同步互访，Promise 微任务和 MutationObserver 混合操作；与 HTML 规范 oracle 对照 | 任务源/微任务次序一致；live DOM 不出现跨 Realm 并行写 | 待实现 | 未执行 |
| `RED-BD-020` | R2-R3 | 固定 DOM 仅修改 scroll、CSS animation、focus、overlay 后提交旧坐标与 locator | StrictCAS 拒绝旧依赖；DynamicLocator 在真实 hit-test 通过后才派发 | 待实现 | 未执行 |
| `RED-BD-021` | R2-R3 | 在等待按钮期间多次 React/Vue style 重渲染替换同名按钮 | 只能在首次派发前重定位；提交事件至多发送一次 | 待实现 | 未执行 |
| `RED-BD-022` | R1 | read_only 页面自动执行 POST/Beacon/WS，注入 TCP 断线与重试 | Broker 默认拒绝未获权写；未知结果记录 uncertain，无二次提交 | 待实现 | 未执行 |
| `RED-BD-023` | R2-R3 | 同一幂等键分别提交相同负载、不同摘要，并注入 202 回执丢失 | 同摘要返回原 action；不同摘要 409；状态查询仍可用 | `crates/aiweb-core/tests/red_regressions.rs`（部分） | 未执行 |
| `RED-BD-024` | R1 | 以 tenant-A 凭据伪造 tenant-B body identity 与对象 ID | 无跨租户可见/操作；审计归属真实 principal | `crates/aiweb-core/tests/red_regressions.rs`（部分） | 未执行 |
| `RED-BD-025` | R1 | tenant-A 页面写入 cookie/cache/JS memory 标记，再销毁并创建 tenant-B | 活动脚本进程不跨租户复用，0 残留/密钥泄漏 | 待实现 | 未执行 |
| `RED-BD-026` | R1 | cross-site iframe、敏感 JSON、CORS preflight、DNS rebinding、重定向和代理升级 | 不允许越权敏感响应分发或禁止 IP 的连接 | 待实现 | 未执行 |
| `RED-BD-027` | R1 | 禁止策略/监控初始化后启动 R0 控制面 | API fail closed，基础 auth、audit、metrics 可观测 | 待实现 | 未执行 |
| `RED-BD-028` | R2-R3 | 同任务 2 已确认 effect + 1 网络已发未确认 effect，然后取消任务 | 逐动作账本不丢，任务为 Cancelled + Uncertain | `crates/aiweb-core/tests/red_regressions.rs`（部分） | 未执行 |
| `RED-BD-029` | R2-R3 | JS while(true)、FD/PID 疲劳与强制 OOM，并取消请求 | Supervisor 进程树强制回收，其它租户不受影响 | 待实现 | 未执行 |
| `RED-BD-030` | R2-R3 | Page 创建后多次 navigate，构造 SSE seq Gap 与游标过期 | 能按导航 ID 查状态；EVENT_GAP 后租户授权全量恢复 | 待实现 | 未执行 |
| `RED-BD-031` | R2-R3 | Native 98% 通过 vs Chromium 100% + Native 1.2x 吞吐 | 非劣性 Gate 拒绝等价吞吐结论，失败全部公开 | 待实现 | 未执行 |
| `RED-BD-032` | R2-R3 | 删除设计到测试的部分 PLAN 映射并伪造状态为通过 | 静态/CI Gate 失败，真实测试无证据不可通过 | 待实现 | 未执行 |
| `RED-BD-033` | R2-R3 | 两个 test id 使用同一服务器账号、同 DB 行写入 | 测试命名空间管理发现冲突/阻断，清理不可遗漏 | 待实现 | 未执行 |
| `RED-BD-034` | R2-R3 | 构造共享 file-backed pages 和 PSS/RSS 不一致 | 明确分列进程树 RSS sum/PSS/cgroup 与计量方法 | 待实现 | 未执行 |

## 12. 22 项详细红测（编号保持不变）

| 红测 ID | P 级 | 对抗场景 | DD 控制 | 脚本 | 状态 |
|---|---|---|---|---|---|
| `RED-DD-001` | P0 | 表单值绕过 DOM 变更版本 | DD-014, DD-068 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-002` | P0 | 一次 Action 可触发多条外部效果 | DD-022, DD-072, DD-073 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-003` | P0 | 失效租约 Worker 仍向外发送 | DD-010, DD-027, DD-072 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-004` | P0 | 跨源响应与凭证错配 | DD-015, DD-047, DD-078 | 待实现 | 未执行 |
| `RED-DD-005` | P0 | 共享缓存泄露认证内容 | DD-011, DD-030, DD-077 | 待实现 | 未执行 |
| `RED-DD-006` | P0 | 快照一致性被误认为跨 Frame 原子 | DD-013, DD-067, DD-034 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-007` | P0 | Playwright 风格 Locator 伪兼容 | DD-020, DD-026, DD-069 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-008` | P0 | 确认级别误导业务成功 | DD-073, DD-074, DD-033 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-009` | P0 | 审批绕过跳转及动态目标 | DD-050, DD-048, DD-076 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-010` | P0 | 同页并发写事件序列被错误并行化 | DD-012, DD-017, DD-067 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-011` | P1 | 多流 SSE 光标丢失事件 | DD-024, DD-037, DD-075 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-012` | P1 | 单调时钟在重启/迁移后不可比较 | DD-010, DD-027, DD-034 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-013` | P1 | SQL 跨租户外键与清理缺口 | DD-032, DD-035, DD-073 | 待实现 | 未执行 |
| `RED-DD-014` | P1 | IPC 反序列化与伪造 Renderer 身份 | DD-004, DD-028, DD-076 | 待实现 | 未执行 |
| `RED-DD-015` | P1 | 无限流请求绕过预算 | DD-023, DD-071, DD-072 | `crates/aiweb-core/tests/red_regressions.rs`（仅 Fixture 断言） | 未执行 |
| `RED-DD-016` | P1 | 资源成本统计漏算控制面与并行公共服务 | DD-023, DD-054, DD-079 | 待实现 | 未执行 |
| `RED-DD-017` | P1 | 按需绘制时帧推进与可操作性等待死锁 | DD-003, DD-018, DD-069 | 待实现 | 未执行 |
| `RED-DD-018` | P1 | Playwright 等待条件宽泛且易产生假通过 | DD-015, DD-026, DD-063 | 待实现 | 未执行 |
| `RED-DD-019` | P1 | 安全门槛和兼容分母被选择性稀释 | DD-053, DD-059, DD-080 | 待实现 | 未执行 |
| `RED-DD-020` | P1 | Prompt Injection 通过结构化摘要提权 | DD-049, DD-050, DD-051 | 待实现 | 未执行 |
| `RED-DD-021` | P2 | 测试隔离仅限本地服务端 | DD-029, DD-061, DD-063 | 待实现 | 未执行 |
| `RED-DD-022` | P2 | Snapshot/证据保留与复用误读 | DD-013, DD-025, DD-034 | 待实现 | 未执行 |

## 13. 测试证据与追踪

追踪矩阵 `Test_Traceability_v1.0.xlsx` 维护 157 项需求 → 81 项 DD → 56 项群组/红测计划以及已编写代码的定位；尚未形成逐需求实测保证。`tests/results/test_registry.json` 是机器可读索引。每一次未来实际测试必须新增 immutable run_id、源代码 SHA、测试环境 digest、真实退出码和日志，不允许修改静态计划替代证据。

## 14. 自动化执行方法

```bash
bash tests/scripts/run_all.sh
python3 tests/scripts/validate_traceability.py
# 安装 Playwright 后，仅运行 Chrome Oracle：
cd tests/playwright && npm install && npx playwright install chromium && npm test
```

Windows 可运行 `tests/scripts/run_all.ps1`。未安装 cargo 时脚本明确以非零退出码中止。`tests/scripts/release_gate.py` 会因缺乏 P0 真实证据保持失败关闭。

## 15. 实施与审查结论

详细测试设计与脚手架已生成，尚未完成真实 Rust 编译/运行、Linux 沙箱 SEC、完整 WPT、Native vs Chromium A/B。此交付为受控研发基线，禁止用于不可信公网浏览。
