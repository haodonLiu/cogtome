set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

# 构建（WebUI + 二进制）
build:
    cd webui && npm ci && npm run build
    cargo build --release
    @echo "✓ target/release/cogtome"

# 启动 WebUI + API
start *ARGS:
    cargo run --release -- serve {{ARGS}}

# 运行 skill / unit / motif
run NAME *ARGS:
    cargo run --release -- run {{NAME}} {{ARGS}}

# 测试
test:
    cargo test
    cd webui && npm run test:run

# 打 Linux .deb
deb: build
    bash packaging/deb/build-deb.sh

# 打 Windows 安装包
win: build
    bash packaging/windows/build-installer.sh

# 清理构建产物
clean:
    cargo clean
    cd webui && rm -rf dist node_modules
    rm -rf target/deb-staging target/windows-staging
