#!/bin/bash

# Claude Night Pilot - 自動化清理腳本
# 根據分析結果執行安全的專案清理

set -e

echo "🧹 Claude Night Pilot 自動化清理腳本"
echo "生成時間: $(date)"
echo "═══════════════════════════════════════"

# 顏色定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 安全模式標誌
SAFE_MODE="${SAFE_MODE:-true}"
DRY_RUN="${DRY_RUN:-false}"

# 清理統計
CLEANED_FILES=0
CLEANED_SIZE=0
SKIPPED_FILES=0

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

check_prerequisites() {
    log_info "檢查前置條件..."
    
    # 確認在正確的專案目錄
    if [[ ! -f "package.json" ]] || [[ ! -d "src-tauri" ]]; then
        log_error "錯誤：請在 Claude Night Pilot 專案根目錄執行此腳本"
        exit 1
    fi
    
    # 檢查 Git 狀態
    if [[ $(git status --porcelain | wc -l) -gt 0 ]]; then
        log_warning "警告：Git 工作區不乾淨，建議先提交變更"
        if [[ "$SAFE_MODE" == "true" ]]; then
            read -p "是否繼續？ (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "清理已取消"
                exit 0
            fi
        fi
    fi
    
    log_success "前置條件檢查完成"
}

cleanup_build_artifacts() {
    log_info "清理編譯產物..."
    
    # Rust target 目錄
    if [[ -d "src-tauri/target" ]]; then
        local size=$(du -sh src-tauri/target | cut -f1)
        log_info "發現 Rust target 目錄 (大小: $size)"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_warning "[DRY RUN] 將會刪除: src-tauri/target/"
        else
            cd src-tauri
            cargo clean
            cd ..
            log_success "已清理 Rust target 目錄"
            ((CLEANED_FILES++))
        fi
    fi
    
    # Node modules (如果存在)
    if [[ -d "node_modules" ]]; then
        local size=$(du -sh node_modules | cut -f1)
        log_info "發現 node_modules 目錄 (大小: $size)"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_warning "[DRY RUN] 將會刪除: node_modules/"
        else
            rm -rf node_modules
            log_success "已清理 node_modules 目錄"
            ((CLEANED_FILES++))
        fi
    fi
    
    # 清理其他編譯產物
    local temp_files=(
        "dist/"
        "build/"
        ".next/"
        "coverage/"
    )
    
    for temp_file in "${temp_files[@]}"; do
        if [[ -d "$temp_file" ]]; then
            if [[ "$DRY_RUN" == "true" ]]; then
                log_warning "[DRY RUN] 將會刪除: $temp_file"
            else
                rm -rf "$temp_file"
                log_success "已清理: $temp_file"
                ((CLEANED_FILES++))
            fi
        fi
    done
}

cleanup_archive_files() {
    log_info "清理 archive 目錄..."
    
    if [[ -d "archive" ]]; then
        local file_count=$(find archive -type f | wc -l)
        local size=$(du -sh archive | cut -f1)
        
        log_info "Archive 目錄包含 $file_count 個檔案 (大小: $size)"
        
        if [[ "$SAFE_MODE" == "true" ]]; then
            log_warning "安全模式：保留 archive 目錄"
            log_info "如需刪除，請手動執行: rm -rf archive/"
            ((SKIPPED_FILES += file_count))
        else
            if [[ "$DRY_RUN" == "true" ]]; then
                log_warning "[DRY RUN] 將會刪除 archive 目錄"
            else
                rm -rf archive/
                log_success "已清理 archive 目錄"
                ((CLEANED_FILES += file_count))
            fi
        fi
    else
        log_info "未發現 archive 目錄"
    fi
}

cleanup_duplicate_cli() {
    log_info "清理重複的 CLI 實現..."
    
    local unified_cli="src-tauri/src/bin/cnp-unified.rs"
    local optimized_cli="src-tauri/src/bin/cnp-optimized.rs"
    
    if [[ -f "$unified_cli" ]] && [[ -f "$optimized_cli" ]]; then
        log_info "發現重複的 CLI 實現"
        log_info "保留: $optimized_cli (性能優化版)"
        log_info "移除: $unified_cli"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_warning "[DRY RUN] 將會刪除: $unified_cli"
        else
            if [[ "$SAFE_MODE" == "true" ]]; then
                # 備份而非刪除
                mv "$unified_cli" "${unified_cli}.backup"
                log_success "已備份: ${unified_cli}.backup"
            else
                rm "$unified_cli"
                log_success "已刪除重複的 CLI 實現"
            fi
            ((CLEANED_FILES++))
        fi
    else
        log_info "未發現重複的 CLI 實現"
    fi
}

cleanup_temporary_files() {
    log_info "清理臨時檔案..."
    
    # 常見的臨時檔案模式
    local temp_patterns=(
        "*.tmp"
        "*.temp"
        "*~"
        ".DS_Store"
        "Thumbs.db"
        "*.log"
        "*.swp"
        "*.swo"
    )
    
    for pattern in "${temp_patterns[@]}"; do
        local files=($(find . -name "$pattern" -not -path "./node_modules/*" -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true))
        
        if [[ ${#files[@]} -gt 0 ]]; then
            log_info "發現 ${#files[@]} 個 $pattern 檔案"
            
            for file in "${files[@]}"; do
                if [[ "$DRY_RUN" == "true" ]]; then
                    log_warning "[DRY RUN] 將會刪除: $file"
                else
                    rm -f "$file"
                    ((CLEANED_FILES++))
                fi
            done
            
            if [[ "$DRY_RUN" != "true" ]]; then
                log_success "已清理 $pattern 檔案"
            fi
        fi
    done
}

optimize_git_repository() {
    log_info "優化 Git 儲存庫..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "[DRY RUN] 將會執行 Git 優化"
        return
    fi
    
    # Git 垃圾回收
    git gc --aggressive --prune=now
    log_success "Git 垃圾回收完成"
    
    # 清理未追蹤的檔案 (謹慎操作)
    if [[ "$SAFE_MODE" != "true" ]]; then
        git clean -fd
        log_success "已清理未追蹤的檔案"
    else
        log_warning "安全模式：跳過清理未追蹤檔案"
        log_info "如需清理，請手動執行: git clean -fd"
    fi
}

update_package_lock() {
    log_info "更新套件鎖定檔案..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "[DRY RUN] 將會更新套件鎖定檔案"
        return
    fi
    
    # 重新生成 package-lock.json (如果存在 package.json)
    if [[ -f "package.json" ]] && command -v npm &> /dev/null; then
        npm install --package-lock-only
        log_success "已更新 package-lock.json"
    fi
    
    # 重新生成 Cargo.lock
    if [[ -f "src-tauri/Cargo.toml" ]]; then
        cd src-tauri
        cargo generate-lockfile
        cd ..
        log_success "已更新 Cargo.lock"
    fi
}

generate_report() {
    echo ""
    echo "📊 清理報告"
    echo "═══════════════════════════════════════"
    echo "清理檔案數: $CLEANED_FILES"
    echo "跳過檔案數: $SKIPPED_FILES"
    echo "模式: $(if [[ "$DRY_RUN" == "true" ]]; then echo "DRY RUN"; elif [[ "$SAFE_MODE" == "true" ]]; then echo "SAFE"; else echo "NORMAL"; fi)"
    echo "完成時間: $(date)"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo ""
        log_info "這是 DRY RUN 模式，沒有實際刪除檔案"
        log_info "如需執行真正的清理，請執行: DRY_RUN=false $0"
    fi
    
    echo ""
    log_success "清理作業完成！"
}

# 主要執行流程
main() {
    echo ""
    log_info "開始執行清理作業..."
    echo "安全模式: $SAFE_MODE"
    echo "DRY RUN 模式: $DRY_RUN"
    echo ""
    
    check_prerequisites
    
    cleanup_build_artifacts
    cleanup_temporary_files
    cleanup_duplicate_cli
    
    # 較危險的清理操作
    if [[ "$SAFE_MODE" != "true" ]]; then
        cleanup_archive_files
        optimize_git_repository
    else
        log_warning "安全模式：跳過 archive 清理和 Git 優化"
    fi
    
    update_package_lock
    generate_report
}

# 處理命令行參數
case "${1:-}" in
    --dry-run)
        DRY_RUN=true
        SAFE_MODE=false
        ;;
    --unsafe)
        SAFE_MODE=false
        ;;
    --help|-h)
        echo "Claude Night Pilot 自動化清理腳本"
        echo ""
        echo "用法: $0 [選項]"
        echo ""
        echo "選項:"
        echo "  --dry-run    執行 DRY RUN 模式，不實際刪除檔案"
        echo "  --unsafe     關閉安全模式，執行所有清理操作"
        echo "  --help       顯示此說明訊息"
        echo ""
        echo "環境變數:"
        echo "  SAFE_MODE    設為 false 以關閉安全模式"
        echo "  DRY_RUN      設為 true 以啟用 DRY RUN 模式"
        echo ""
        exit 0
        ;;
esac

# 執行主程式
main

exit 0