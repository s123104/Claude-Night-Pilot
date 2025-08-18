#!/usr/bin/env bash
set -euo pipefail

# 簡介：將本倉庫的 mcp-servers.global.json 安裝為 Codex CLI 的全域 MCP 設定。
# 注意：不同客戶端的全域設定路徑可能不同。此腳本會嘗試以下路徑：
#  - ~/.config/codex/mcp.json
#  - ~/.config/codex-cli/mcp.json
#  - ~/.codex/mcp.json
# 若您的 Codex CLI 有特定路徑或支援 --mcp-config 旗標，亦可直接指定檔案路徑。

CONFIG_SRC="$(cd "$(dirname "$0")/.." && pwd)/mcp-servers.global.json"

declare -a TARGETS=(
  "$HOME/.config/codex/mcp.json"
  "$HOME/.config/codex-cli/mcp.json"
  "$HOME/.codex/mcp.json"
)

echo "來源設定：$CONFIG_SRC"

for target in "${TARGETS[@]}"; do
  dir="$(dirname "$target")"
  mkdir -p "$dir"
  cp "$CONFIG_SRC" "$target"
  echo "已安裝：$target"
done

cat <<'EOF'

提示：
- 若 Codex CLI 支援參數，亦可顯式指定：
  codex --mcp-config /path/to/mcp-servers.global.json

相依需求（請依實際情況安裝）：
- npx（Node.js 18+）
- docker（且容器：docker-site-mcp-memory、docker-site-mcp-everything 已運行）
- uv / uvx（Python 套件管理工具）

驗證：
- 啟動 MCP 客戶端後確認各伺服器可用，或以 CLI 列出可用工具。
EOF

