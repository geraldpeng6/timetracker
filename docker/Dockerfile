# 多阶段构建 Dockerfile
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libx11-dev \
    libxrandr-dev \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /usr/src/app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 来缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 构建依赖（这一层会被缓存）
RUN cargo build --release && rm -rf src

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libxrandr2 \
    libxi6 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 timetracker

# 复制二进制文件
COPY --from=builder /usr/src/app/target/release/timetracker /usr/local/bin/timetracker

# 设置权限
RUN chmod +x /usr/local/bin/timetracker

# 切换到非 root 用户
USER timetracker

# 设置工作目录
WORKDIR /home/timetracker

# 暴露数据目录
VOLUME ["/home/timetracker/.timetracker"]

# 默认命令
CMD ["timetracker", "--help"]