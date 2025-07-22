# TimeTracker Docker 镜像
# 多阶段构建，支持多架构

# 构建阶段
FROM rust:1.75-slim as builder

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libxcb-randr0-dev \
    libxcb-xtest0-dev \
    libxcb-xinerama0-dev \
    libxcb-shape0-dev \
    libxcb-xkb-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /usr/src/timetracker

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 以缓存依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 构建依赖（缓存层）
RUN cargo build --release && rm -rf src

# 复制源代码
COPY src ./src
COPY docs ./docs

# 构建应用
RUN cargo build --release

# 运行时阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libxcb1 \
    libxcb-randr0 \
    libxcb-xtest0 \
    libxcb-xinerama0 \
    libxcb-shape0 \
    libxcb-xkb1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 timetracker

# 复制二进制文件
COPY --from=builder /usr/src/timetracker/target/release/timetracker /usr/local/bin/timetracker

# 复制文档
COPY --from=builder /usr/src/timetracker/docs /usr/share/doc/timetracker/
COPY README.md LICENSE /usr/share/doc/timetracker/

# 设置权限
RUN chmod +x /usr/local/bin/timetracker

# 创建数据目录
RUN mkdir -p /home/timetracker/.timetracker && \
    chown -R timetracker:timetracker /home/timetracker

# 切换到非 root 用户
USER timetracker
WORKDIR /home/timetracker

# 设置环境变量
ENV RUST_LOG=info
ENV TIMETRACKER_DATA_DIR=/home/timetracker/.timetracker

# 暴露端口（如果有 web 界面）
# EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD timetracker status || exit 1

# 默认命令
CMD ["timetracker", "--help"]

# 标签
LABEL org.opencontainers.image.title="TimeTracker"
LABEL org.opencontainers.image.description="Cross-platform CLI tool for tracking application window usage time with intelligent activity detection"
LABEL org.opencontainers.image.url="https://github.com/geraldpeng6/timetracker"
LABEL org.opencontainers.image.source="https://github.com/geraldpeng6/timetracker"
LABEL org.opencontainers.image.version="0.2.2"
LABEL org.opencontainers.image.licenses="MIT"
