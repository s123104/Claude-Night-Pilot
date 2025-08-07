#!/usr/bin/env python3
"""
測試prompt CRUD功能的Python腳本
使用Claude Code CLI工具進行測試
"""

import subprocess
import json
import time
import sys
from pathlib import Path

def run_cli_command(command_args, capture_output=True, timeout=30):
    """執行CLI命令並返回結果"""
    try:
        cmd = ["cargo", "run", "--bin", "cnp-unified", "--"] + command_args
        print(f"🚀 執行命令: {' '.join(cmd)}")
        
        result = subprocess.run(
            cmd, 
            capture_output=capture_output, 
            text=True, 
            timeout=timeout,
            cwd="/Users/azlife.eth/Claude-Night‑Pilot/src-tauri"
        )
        
        if result.returncode == 0:
            print(f"✅ 命令執行成功")
            if capture_output and result.stdout:
                return result.stdout.strip()
            return True
        else:
            print(f"❌ 命令執行失敗: {result.stderr}")
            return None
            
    except subprocess.TimeoutExpired:
        print("⏰ 命令執行超時")
        return None
    except Exception as e:
        print(f"❌ 執行錯誤: {e}")
        return None

def test_immediate_execution():
    """測試立即執行功能"""
    print("\n📋 測試1: 立即執行功能")
    
    # 測試立即執行
    result = run_cli_command([
        "execute", 
        "--prompt", "這是一個立即執行的測試",
        "--mode", "sync",
        "--format", "json"
    ])
    
    if result:
        try:
            response = json.loads(result)
            print(f"✅ 立即執行成功")
            print(f"📊 回應長度: {len(response.get('completion', ''))}")
            if 'execution_metadata' in response:
                metadata = response['execution_metadata']
                print(f"🕐 執行時間: {metadata.get('total_attempts', 'N/A')} 次嘗試")
            return True
        except json.JSONDecodeError:
            print("❌ 回應格式錯誤")
            return False
    return False

def test_cooldown_detection():
    """測試冷卻檢測功能"""
    print("\n❄️ 測試2: 冷卻檢測功能")
    
    result = run_cli_command(["cooldown", "--format", "json"])
    
    if result:
        try:
            cooldown_info = json.loads(result)
            is_cooling = cooldown_info.get('is_cooling', False)
            print(f"🕐 系統冷卻狀態: {'冷卻中' if is_cooling else '可用'}")
            
            if is_cooling:
                remaining = cooldown_info.get('seconds_remaining', 0)
                print(f"⏰ 剩餘冷卻時間: {remaining} 秒")
            
            return True
        except json.JSONDecodeError:
            print("❌ 冷卻檢測回應格式錯誤")
            return False
    return False

def test_health_check():
    """測試系統健康檢查"""
    print("\n🏥 測試3: 系統健康檢查")
    
    result = run_cli_command(["health", "--format", "pretty"], capture_output=False)
    
    return result is not None

def test_batch_execution():
    """測試批量執行功能"""
    print("\n📦 測試4: 批量執行功能")
    
    # 創建測試批量文件
    batch_prompts = [
        {
            "id": "test_prompt_1",
            "prompt": "測試批量執行 - 第1個prompt"
        },
        {
            "id": "test_prompt_2", 
            "prompt": "測試批量執行 - 第2個prompt"
        }
    ]
    
    batch_file = Path("/Users/azlife.eth/Claude-Night‑Pilot/test_batch_prompts.json")
    with open(batch_file, "w", encoding="utf-8") as f:
        json.dump(batch_prompts, f, ensure_ascii=False, indent=2)
    
    print(f"📝 創建批量文件: {batch_file}")
    
    # 執行批量測試
    result = run_cli_command([
        "batch",
        "--file", str(batch_file),
        "--concurrent", "1",
        "--format", "json"
    ], timeout=60)
    
    if result:
        try:
            results = json.loads(result)
            if isinstance(results, list):
                success_count = len([r for r in results if r.get('status') == 'success'])
                print(f"✅ 批量執行完成: {success_count}/{len(results)} 成功")
                return True
        except json.JSONDecodeError:
            print("❌ 批量執行回應格式錯誤")
    
    return False

def test_scheduled_execution():
    """測試排程執行功能"""
    print("\n⏰ 測試5: 排程執行功能")
    
    # 測試async模式（簡化版排程）
    result = run_cli_command([
        "execute",
        "--prompt", "這是一個排程執行測試",
        "--mode", "async", 
        "--format", "pretty"
    ], timeout=30)
    
    return result is not None

def main():
    """主測試函數"""
    print("🚀 開始CLI工具完整測試")
    print("=" * 50)
    
    test_results = []
    
    # 執行所有測試
    tests = [
        ("立即執行功能", test_immediate_execution),
        ("冷卻檢測功能", test_cooldown_detection), 
        ("系統健康檢查", test_health_check),
        ("批量執行功能", test_batch_execution),
        ("排程執行功能", test_scheduled_execution)
    ]
    
    for test_name, test_func in tests:
        try:
            result = test_func()
            test_results.append((test_name, result))
            print(f"\n{'✅' if result else '❌'} {test_name}: {'通過' if result else '失敗'}")
        except Exception as e:
            print(f"\n❌ {test_name}: 測試異常 - {e}")
            test_results.append((test_name, False))
        
        # 測試間隔
        time.sleep(2)
    
    # 輸出測試總結
    print("\n" + "=" * 50)
    print("📊 測試總結")
    print("=" * 50)
    
    passed_count = sum(1 for _, result in test_results if result)
    total_count = len(test_results)
    
    for test_name, result in test_results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status} {test_name}")
    
    print(f"\n總計: {passed_count}/{total_count} 測試通過")
    
    if passed_count == total_count:
        print("🎉 所有測試都通過了！")
        sys.exit(0)
    else:
        print("⚠️ 部分測試失敗，需要進一步調查")
        sys.exit(1)

if __name__ == "__main__":
    main()