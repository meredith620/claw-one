# Claw One 安装测试容器
FROM debian:bookworm-slim

# 安装基础依赖
RUN apt-get update && apt-get install -y \
    curl \
    git \
    systemd \
    procps \
    lsof \
    && rm -rf /var/lib/apt/lists/*

# 创建测试用户
RUN useradd -m -s /bin/bash testuser

# 设置工作目录
WORKDIR /test

# 复制分发包 (从构建上下文根目录)
COPY claw-one-d9fa896-x86_64.tar.gz /test/

# 解压分发包
RUN tar xzf claw-one-d9fa896-x86_64.tar.gz

# 设置权限
RUN chown -R testuser:testuser /test

# 切换到测试用户
USER testuser
ENV HOME=/home/testuser

# 设置工作目录到分发包
WORKDIR /test/claw-one-d9fa896-x86_64

# 默认命令：运行环境检查并安装
CMD ["bash", "-c", "echo '=== 环境检查 ===' && ./scripts/check-env.sh && echo '' && echo '=== 开始安装 ===' && ./scripts/install.sh"]
