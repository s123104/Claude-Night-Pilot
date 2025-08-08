#!/usr/bin/env python3

"""
Claude Night Pilot 快速性能測試
測量關鍵性能指標並與目標進行比較
"""

import subprocess
import time
import json
import statistics
import os
from pathlib import Path

class PerformanceTester:
    def __init__(self):
        self.cli_path = "./target/release/cnp-unified"
        self.results = {}
        
    def run_command_with_timing(self, cmd, iterations=5):
        """執行命令並測量時間"""
        times = []
        for _ in range(iterations):
            start = time.time()
            try:
                result = subprocess.run(
                    cmd, 
                    shell=True, 
                    capture_output=True, 
                    text=True,
                    timeout=10
                )
                end = time.time()
                execution_time = end - start
                times.append(execution_time)
                
                # 第一次迭代記錄輸出大小
                if _ == 0:
                    output_size = len(result.stdout) + len(result.stderr)
                    return times, result.returncode == 0, output_size
                    
            except subprocess.TimeoutExpired:
                times.append(10.0)  # Timeout 作為最大值
                
        return times, True, 0
    
    def test_cli_startup(self):
        """測試 CLI 啟動時間"""
        print("🚀 測試 CLI 啟動性能...")
        
        # Help 命令 (最快的命令)
        times, success, size = self.run_command_with_timing(f"{self.cli_path} --help")
        
        self.results["cli_startup"] = {
            "command": "help",
            "avg_time": statistics.mean(times),
            "min_time": min(times),
            "max_time": max(times),
            "median_time": statistics.median(times),
            "success": success,
            "output_size": size,
            "target": 0.1,  # 100ms 目標
            "meets_target": statistics.median(times) < 0.1
        }
        
    def test_cli_commands(self):
        """測試各種 CLI 命令性能"""
        print("⚡ 測試 CLI 命令性能...")
        
        commands = [
            ("health", f"{self.cli_path} health --format json"),
            ("cooldown", f"{self.cli_path} cooldown --format json"),
        ]
        
        for cmd_name, cmd in commands:
            times, success, size = self.run_command_with_timing(cmd, iterations=3)
            
            self.results[f"cli_{cmd_name}"] = {
                "command": cmd_name,
                "avg_time": statistics.mean(times),
                "min_time": min(times),
                "max_time": max(times),
                "median_time": statistics.median(times),
                "success": success,
                "output_size": size,
                "target": 1.0,  # 1s 目標 (這些命令較複雜)
                "meets_target": statistics.median(times) < 1.0
            }
    
    def test_binary_size(self):
        """測試二進制文件大小"""
        print("📦 檢查二進制文件大小...")
        
        if os.path.exists(self.cli_path):
            size_bytes = os.path.getsize(self.cli_path)
            size_mb = size_bytes / (1024 * 1024)
            
            self.results["binary_size"] = {
                "size_bytes": size_bytes,
                "size_mb": round(size_mb, 2),
                "target_mb": 10,
                "meets_target": size_mb < 10
            }
        else:
            print(f"⚠️ 二進制文件不存在: {self.cli_path}")
    
    def test_cold_vs_warm_start(self):
        """測試冷啟動 vs 熱啟動性能"""
        print("❄️ 測試冷啟動 vs 熱啟動...")
        
        # 清理系統緩存 (macOS)
        subprocess.run("sudo purge", shell=True, capture_output=True)
        
        # 冷啟動
        cold_times, _, _ = self.run_command_with_timing(f"{self.cli_path} --help", iterations=1)
        
        # 熱啟動
        warm_times, _, _ = self.run_command_with_timing(f"{self.cli_path} --help", iterations=3)
        
        self.results["startup_comparison"] = {
            "cold_start": cold_times[0] if cold_times else 0,
            "warm_start_avg": statistics.mean(warm_times) if warm_times else 0,
            "improvement": cold_times[0] - statistics.mean(warm_times) if cold_times and warm_times else 0
        }
    
    def run_all_tests(self):
        """執行所有性能測試"""
        print("🔥 Claude Night Pilot 性能基準測試")
        print("=" * 50)
        
        self.test_binary_size()
        self.test_cli_startup()
        self.test_cli_commands()
        self.test_cold_vs_warm_start()
        
        return self.results
    
    def generate_report(self):
        """生成性能報告"""
        print("\n📊 性能測試結果")
        print("=" * 50)
        
        # 二進制大小
        if "binary_size" in self.results:
            bs = self.results["binary_size"]
            status = "✅" if bs["meets_target"] else "❌"
            print(f"\n📦 二進制大小")
            print(f"  {status} 大小: {bs['size_mb']} MB (目標: <{bs['target_mb']} MB)")
        
        # CLI 性能
        print(f"\n⚡ CLI 性能")
        for key, result in self.results.items():
            if key.startswith("cli_"):
                status = "✅" if result["meets_target"] else "❌"
                print(f"  {status} {result['command']}: {result['median_time']:.3f}s (目標: <{result['target']}s)")
                print(f"      範圍: {result['min_time']:.3f}s - {result['max_time']:.3f}s")
        
        # 啟動比較
        if "startup_comparison" in self.results:
            sc = self.results["startup_comparison"]
            print(f"\n❄️ 啟動性能")
            print(f"  冷啟動: {sc['cold_start']:.3f}s")
            print(f"  熱啟動: {sc['warm_start_avg']:.3f}s")
            print(f"  改善: {sc['improvement']:.3f}s")
        
        # 總體評估
        print(f"\n🎯 總體評估")
        total_tests = 0
        passed_tests = 0
        
        for key, result in self.results.items():
            if isinstance(result, dict) and "meets_target" in result:
                total_tests += 1
                if result["meets_target"]:
                    passed_tests += 1
        
        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0
        print(f"  通過率: {passed_tests}/{total_tests} ({success_rate:.1f}%)")
        
        if success_rate >= 80:
            print("  🎉 性能優秀！")
        elif success_rate >= 60:
            print("  👍 性能良好，有改善空間")
        else:
            print("  ⚠️ 需要性能優化")
    
    def save_results(self, filename="performance_results.json"):
        """保存詳細結果到 JSON 文件"""
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(self.results, f, indent=2, ensure_ascii=False)
        print(f"\n💾 詳細結果已保存到: {filename}")

def main():
    tester = PerformanceTester()
    
    # 檢查二進制文件是否存在
    if not os.path.exists(tester.cli_path):
        print(f"❌ 找不到 CLI 二進制文件: {tester.cli_path}")
        print("請先運行: cargo build --release --bin cnp-unified")
        return
    
    # 執行測試
    results = tester.run_all_tests()
    
    # 生成報告
    tester.generate_report()
    
    # 保存結果
    tester.save_results()

if __name__ == "__main__":
    main()