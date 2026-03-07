# Claw One 安装测试镜像
FROM ubuntu:22.04

# 安装基本依赖
RUN apt-get update && apt-get install -y \
    git \
    curl \
    lsof \
    systemd \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# 创建测试用户
RUN useradd -m -s /bin/bash testuser && \
    echo "testuser:testpass" | chpasswd && \
    usermod -aG sudo testuser

# 设置工作目录
WORKDIR /tmp/claw-one-test

# 复制安装包（构建时复制）
COPY claw-one-*.tar.gz /tmp/

# 切换到测试用户
USER testuser
ENV HOME=/home/testuser

# 默认命令
CMD ["/bin/bash"]
