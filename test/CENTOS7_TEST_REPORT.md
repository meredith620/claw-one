# CentOS 7 musl 兼容性测试报告

**测试时间**: 2026-03-08  
**测试环境**: CentOS 7 (glibc 2.17)  
**测试版本**: claw-one 0.1.0

---

## 测试目的

验证 musl 静态链接二进制在旧版本 Linux 发行版上的兼容性。

---

## 测试环境

```
操作系统: CentOS 7
glibc 版本: 2.17
容器镜像: centos:7
```

---

## 测试结果

### 测试 1: musl 静态链接版本 ✅ 通过

```bash
$ /tmp/claw-one --version
claw-one 0.1.0
```

**结果**: ✅ 成功运行，无依赖错误

**文件信息**:
- 大小: 8.1MB
- 类型: 静态链接 (musl libc)
- 兼容性: 不依赖系统 glibc

---

### 测试 2: glibc 动态链接版本 ❌ 失败

```bash
$ /tmp/claw-one-glibc --version
/tmp/claw-one-glibc: /lib64/libm.so.6: version `GLIBC_2.29' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.18' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.25' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.28' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.29' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.30' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.32' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.33' not found
/tmp/claw-one-glibc: /lib64/libc.so.6: version `GLIBC_2.34' not found
```

**结果**: ❌ 失败，需要 glibc 2.18-2.34

**原因**: 动态链接版本在构建时链接了较新的 glibc 符号，CentOS 7 的 glibc 2.17 无法满足

---

## 结论

| 构建方式 | CentOS 7 兼容性 | 原因 |
|----------|----------------|------|
| **musl 静态链接** | ✅ **通过** | 不依赖系统 glibc，完全自包含 |
| glibc 动态链接 | ❌ 失败 | 依赖构建时的高版本 glibc |

**关键发现**:
1. musl 静态链接提供了真正的跨发行版兼容性
2. 可在 CentOS 7、Ubuntu 20.04 等旧系统运行
3. 单文件分发，无需考虑目标系统的库版本

**推荐**: 生产环境使用 musl 静态构建 (`make dist`)
