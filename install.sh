#!/bin/bash

# Claude Night Pilot 快速安裝腳本
# 支援 macOS 和 Linux

set -e

echo "🚀 Claude Night Pilot 安裝程式"
echo "================================"

# 檢查作業系統
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    *)          MACHINE="UNKNOWN:${OS}"
esac

if [ "$MACHINE" = "UNKNOWN:${OS}" ]; then
    echo "❌ 不支援的作業系統: ${OS}"
    exit 1
fi

echo "✅ 偵測到作業系統: ${MACHINE}"

# 檢查是否有 release 二進制檔案
BINARY_PATH="src-tauri/target/release/cnp-unified"
if [ ! -f "$BINARY_PATH" ]; then
    echo "⚠️  找不到 release 版本，正在編譯..."
    
    # 檢查 Rust 是否已安裝
    if ! command -v cargo &> /dev/null; then
        echo "❌ 未找到 Cargo，請先安裝 Rust"
        echo "   安裝命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # 編譯 release 版本
    echo "🔨 編譯 release 版本..."
    cd src-tauri
    cargo build --release --bin cnp-unified
    cd ..
    
    if [ ! -f "$BINARY_PATH" ]; then
        echo "❌ 編譯失敗"
        exit 1
    fi
    echo "✅ 編譯完成"
fi

# 檢查檔案大小
BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
echo "📦 二進制檔案大小: ${BINARY_SIZE}"

# 安裝到系統路徑
INSTALL_PATH="/usr/local/bin/cnp"
echo "📍 安裝路徑: ${INSTALL_PATH}"

# 檢查是否需要 sudo
if [ -w "/usr/local/bin" ]; then
    cp "$BINARY_PATH" "$INSTALL_PATH"
else
    echo "🔐 需要管理員權限來安裝到系統路徑"
    sudo cp "$BINARY_PATH" "$INSTALL_PATH"
fi

# 設定執行權限
if [ -w "/usr/local/bin" ]; then
    chmod +x "$INSTALL_PATH"
else
    sudo chmod +x "$INSTALL_PATH"
fi

echo "✅ 安裝完成！"
echo ""

# 驗證安裝
echo "🧪 驗證安裝..."
if command -v cnp &> /dev/null; then
    echo "✅ CLI 工具已成功安裝"
    echo ""
    echo "📋 可用命令："
    cnp --help
    echo ""
    echo "🔍 系統狀態檢查："
    cnp health --format json
else
    echo "❌ 安裝驗證失敗"
    echo "請檢查 /usr/local/bin 是否在 PATH 中"
    exit 1
fi

echo ""
echo "🎉 Claude Night Pilot 安裝成功！"
echo ""
echo "📚 使用指南："
echo "  cnp --help          # 查看所有命令"
echo "  cnp health          # 系統健康檢查"
echo "  cnp cooldown        # 檢查 Claude CLI 冷卻狀態"
echo "  cnp execute --help  # 查看執行選項"
echo ""
echo "📖 詳細文檔請參考 DEPLOYMENT_GUIDE.md"