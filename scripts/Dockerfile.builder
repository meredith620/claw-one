# Claw One 编译环境镜像
# 包含 Rust、Node.js、musl 工具链等构建依赖
# 镜像命名: claw-one-builder:{VERSION}

FROM rust:alpine3.21

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    zlib-dev \
    zlib-static \
    git \
    nodejs \
    npm \
    pkgconfig \
    ca-certificates

# 安装 musl 目标
RUN rustup target add x86_64-unknown-linux-musl

# 确保 cargo 目录对所有用户可写（支持任意 UID 运行）
RUN mkdir -p /usr/local/cargo/registry /usr/local/cargo/git && \
    chmod -R 777 /usr/local/cargo

# 创建 builder 用户家目录（供 npm 缓存使用）
RUN mkdir -p /home/builder/.npm && \
    chmod -R 777 /home/builder

# 设置工作目录
WORKDIR /build

# 元数据
LABEL maintainer="claw-one" \
      description="Claw One build environment with Rust, Node.js and musl toolchain"

# 默认命令
CMD ["rustc", "--version"]
