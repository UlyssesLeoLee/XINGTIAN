# AIWEB — AI 原生并行 Web 引擎 · Rust 受控原型 v0.1

本项目遵循详细设计 `AIPWE-DD-001 v1.1` 的原则：**AI 专用、任务原生并行、状态一致性、按需视觉计算、安全边界与失败关闭**。

## 交付内容

- Rust Workspace：`crates/aiweb-core` + `apps/aiwebctl`。核心采用 Rust 标准库，不依赖第三方 Crate。
- 设计：`docs/design/Test_Design_UT_IT_ST_v1.0.md`，分 UT、IT、ST、SEC、WPT/DIFF、PERF。
- Excel：`docs/design/Test_Traceability_v1.0.xlsx`，保留 157 条需求、81 项 DD 控制、34 项基本设计计划及 22 项详细设计红测，附 ADR 阻断。
- 测试脚本：`crates/aiweb-core/tests`（Rust），`tests/scripts`（Shell/PowerShell/Python），`tests/playwright`（Chromium 参考行为）。
- 基线：`docs/upstream` 中含要件定义、基本设计、详细设计和详细设计追踪矩阵 v1.1。

## 最小使用步骤（Windows PowerShell）

```powershell
# 安装 Rust 工具链后，在本目录执行
cargo test --workspace --all-targets
cargo run -p aiwebctl -- capabilities
cargo run -p aiwebctl -- demo
.\tests\scripts\run_all.ps1
```

Linux：`bash tests/scripts/run_all.sh`。纯静态追踪检查：`python3 tests/scripts/validate_traceability.py`。

## 原型具备的功能

任务 DAG 验证、限额队列与租户轮询、内存 Fixture 页面快照、表单值版本控制、严格 CAS、简单 Selector 再定位、动作唯一派发记录、内存 EffectLedger、撤销租约的出站许可模拟、事件游标缺口及跨 Boot 时钟拒绝。提供 `5 读 2 写` 的并发回归脚本。

## 严格限制

**当前版本不是 Chrome 兼容浏览器。** 未实现 HTML5 parser、真实 JS 宿主和事件循环、CSS/Layout/截图、真实出站网络、OS 沙箱、持久化数据库、HTTP 服务、Playwright 原生 Driver。所有 `security.rs` 授权都只属于受控进程内模拟，不能构成实际安全保护。不能浏览不可信公网网站。

Playwright 套件运行的浏览器是 Chromium，作用只是建立 Oracle。它无法证明自研 Rust 内核通过任何 Playwright 兼容测试。性能与安全放行也未执行。

## 验收与状态

- 34 项继承群组测试与 22 项新增对抗红测仍为 **未执行**，即使其中已有部分 Fixture 测试源码。
- 编译与 cargo test 需在实际具备 Rust 工具链的开发/CI 环境执行；生成环境没有 rustc/cargo。
- `tests/scripts/release_gate.py` 默认阻断生产发布，不允许用编写测试脚本代替真实运行证据。
- 逐个高风险 ADR 冻结之前，不能把暂定接口表述为稳定公开 API。
