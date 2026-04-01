---
# HARNESS METADATA
# type: specification
# part-of: harness-architecture
# scope: harness-maintenance
# managed-by: harness-system
# version: 1.0
# created: 2026-04-01
---

# Harness 演进规范

> **适用于:** 修改任何 Harness 架构文件
> 
003e ⚠️ **HARNESS 文件**: 本文档属于 Harness 架构，修改需谨慎

## 何时需要修改 Harness

Harness 不是一成不变的，以下情况**应该**修改：

| 场景 | 示例 | 操作 |
|------|------|------|
| 技术栈变更 | 从 Axum 切换到 Actix | 更新 `specs/api.spec.md` |
| 架构演进 | 从单 crate 拆分为多 crate | 更新 `specs/architecture.constraint.md` |
| 新增约束 | 引入新的代码规范 | 在对应 `.constraint.md` 添加 |
| 流程优化 | 改进 CI/CD 流程 | 更新 `specs/release.spec.md` |
| 修复漏洞 | 发现架构验证漏洞 | 更新 `.entropy-guards/` |

## 修改 Harness 的流程

```
1. 识别需求
   ↓ 确认这是 Harness 层面的变更，而非业务功能
2. 创建分支
   ↓ git checkout -b harness/update-api-spec
3. 修改文件
   ↓ 遵循本文档的格式要求
4. 更新版本
   ↓ 修改 harness.yaml 中的 version
5. 更新元数据
   ↓ 修改文件的 HARNESS METADATA 头部
6. 本地验证
   ↓ 运行 ./scripts/harness/validate-arch.sh
7. 提交
   ↓ 提交消息必须包含 "harness" 关键词
8. Review
   ↓ 建议 PR review，尤其是架构变更
9. 合并
   ↓ 更新 AGENTS.md 中的 Last Updated
```

## 文件格式规范

### HARNESS METADATA 头部

每个 Harness 文件必须包含以下 YAML 头部：

```yaml
---
# HARNESS METADATA
# type: [entry-point|specification|constraint|script|manifest|rule]
# part-of: harness-architecture
# scope: [api-development|configuration|architecture|testing|release|...]
# managed-by: harness-system
# do-not-modify-manually: 如需修改请遵循 harness-evolution.spec.md
# version: 1.0
# created: 2026-04-01
---
```

### 类型说明

| type | 用途 | 示例 |
|------|------|------|
| `entry-point` | Agent 入口文档 | AGENTS.md |
| `specification` | 开发规范 | api.spec.md |
| `constraint` | 架构约束 | config.constraint.md |
| `script` | Harness 脚本 | *.sh |
| `manifest` | 配置文件 | harness.yaml |
| `rule` | 熵防护规则 | *.rule |

### 版本管理

- **harness.yaml version**: Harness 整体版本，重大变更时 bump
- **文件内部 version**: 单个文件的版本，独立演进
- **created**: 文件创建日期，不变
- **last-modified**: 可选，最后修改日期

## 提交消息规范

修改 Harness 时，提交消息必须：

```
harness: 简短描述变更内容

详细说明：
- 为什么需要这个变更
- 影响了哪些 Agent 行为
- 是否向后兼容

修改的文件:
- specs/xxx.md
- harness.yaml
```

**关键词要求：**
- 必须包含 `harness`（触发保护规则检查）
- 建议包含 `spec`、`constraint`、`architecture` 等关键词

## 向后兼容性

### 破坏性变更

以下变更需要 bump 主版本号（1.0 → 2.0）：
- 删除已有的约束或规范
- 修改已有 API/脚本的签名
- 改变 Agent 加载 Harness 的方式

### 非破坏性变更

以下变更是向后兼容的（bump 次版本号）：
- 新增规范或约束
- 新增脚本或工具
- 补充文档说明
- 修复 typo

## 最佳实践

1. **渐进式演进**
   - 不要一次性重写整个 Harness
   - 小步迭代，每次只改一个方面

2. **文档先行**
   - 先更新 spec，再改实现
   - 确保 Agent 能先理解新规则

3. **验证驱动**
   - 新增约束？先写验证脚本
   - 新增规范？先写测试用例

4. **保持 DRY**
   - 不要在多个 spec 中重复相同内容
   - 使用引用：`参见 [api.spec.md](api.spec.md)`

## 禁止的操作

⛔ **永远不要：**
- 在没有明确需求的情况下修改 Harness
- 为单个业务功能创建专门的 Harness 文件
- 在 Harness 中硬编码项目特定的实现细节
- 删除没有替代方案的旧约束

---

*本文档是 Harness 的元规范 —— 规范如何修改规范本身。*
