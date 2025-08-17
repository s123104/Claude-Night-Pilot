// 📊 Claude Night Pilot - 企業級指標收集系統
// 基於Context7最佳實踐的效能監控
// 創建時間: 2025-08-17T05:20:00+00:00

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// 企業級指標收集器
/// 
/// 收集指標類型：
/// - 操作指標（響應時間、成功率）
/// - 系統指標（CPU、記憶體）
/// - 業務指標（任務數、執行率）
/// - 錯誤指標（錯誤率、異常類型）
pub struct MetricsCollector {
    /// 操作統計
    operations: HashMap<String, OperationStats>,
    
    /// 錯誤統計
    errors: HashMap<String, ErrorStats>,
    
    /// 系統啟動時間
    start_time: Instant,
    
    /// 總操作數
    total_operations: u64,
    
    /// 總錯誤數
    total_errors: u64,
}

/// 操作統計數據
#[derive(Debug, Clone)]
pub struct OperationStats {
    /// 操作名稱
    pub name: String,
    
    /// 執行次數
    pub count: u64,
    
    /// 成功次數
    pub success_count: u64,
    
    /// 總執行時間
    pub total_duration: Duration,
    
    /// 最小執行時間
    pub min_duration: Duration,
    
    /// 最大執行時間
    pub max_duration: Duration,
    
    /// 最後執行時間
    pub last_execution: Instant,
}

/// 錯誤統計數據
#[derive(Debug, Clone)]
pub struct ErrorStats {
    /// 錯誤類型
    pub error_type: String,
    
    /// 發生次數
    pub count: u64,
    
    /// 首次發生時間
    pub first_occurrence: Instant,
    
    /// 最後發生時間
    pub last_occurrence: Instant,
    
    /// 相關組件
    pub components: HashMap<String, u64>,
}

/// 指標摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// 系統運行時間（秒）
    pub uptime_seconds: u64,
    
    /// 總操作數
    pub total_operations: u64,
    
    /// 總錯誤數
    pub error_count: u64,
    
    /// 成功率 (0.0 - 1.0)
    pub success_rate: f64,
    
    /// 平均響應時間（毫秒）
    pub avg_response_time: f64,
    
    /// 操作統計
    pub operations: Vec<OperationSummary>,
    
    /// 錯誤統計
    pub errors: Vec<ErrorSummary>,
}

/// 操作摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSummary {
    pub name: String,
    pub count: u64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

/// 錯誤摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub error_type: String,
    pub count: u64,
    pub components: HashMap<String, u64>,
}

impl MetricsCollector {
    /// 創建新的指標收集器
    pub fn new() -> Self {
        MetricsCollector {
            operations: HashMap::new(),
            errors: HashMap::new(),
            start_time: Instant::now(),
            total_operations: 0,
            total_errors: 0,
        }
    }
    
    /// 記錄操作指標
    pub fn record_operation(&mut self, operation: &str, duration: Duration, success: bool) {
        self.total_operations += 1;
        
        let stats = self.operations.entry(operation.to_string()).or_insert_with(|| {
            OperationStats {
                name: operation.to_string(),
                count: 0,
                success_count: 0,
                total_duration: Duration::new(0, 0),
                min_duration: duration,
                max_duration: duration,
                last_execution: Instant::now(),
            }
        });
        
        stats.count += 1;
        if success {
            stats.success_count += 1;
        }
        stats.total_duration += duration;
        stats.min_duration = stats.min_duration.min(duration);
        stats.max_duration = stats.max_duration.max(duration);
        stats.last_execution = Instant::now();
    }
    
    /// 記錄錯誤
    pub fn record_error(&mut self, component: &str, error_type: &str) {
        self.total_errors += 1;
        
        let stats = self.errors.entry(error_type.to_string()).or_insert_with(|| {
            ErrorStats {
                error_type: error_type.to_string(),
                count: 0,
                first_occurrence: Instant::now(),
                last_occurrence: Instant::now(),
                components: HashMap::new(),
            }
        });
        
        stats.count += 1;
        stats.last_occurrence = Instant::now();
        *stats.components.entry(component.to_string()).or_insert(0) += 1;
    }
    
    /// 獲取指標摘要
    pub fn get_summary(&self) -> MetricsSummary {
        let uptime = self.start_time.elapsed().as_secs();
        
        let success_count: u64 = self.operations.values()
            .map(|stats| stats.success_count)
            .sum();
        
        let success_rate = if self.total_operations > 0 {
            success_count as f64 / self.total_operations as f64
        } else {
            1.0
        };
        
        let total_duration: Duration = self.operations.values()
            .map(|stats| stats.total_duration)
            .sum();
        
        let avg_response_time = if self.total_operations > 0 {
            total_duration.as_millis() as f64 / self.total_operations as f64
        } else {
            0.0
        };
        
        let operations = self.operations.values()
            .map(|stats| OperationSummary {
                name: stats.name.clone(),
                count: stats.count,
                success_rate: if stats.count > 0 {
                    stats.success_count as f64 / stats.count as f64
                } else {
                    1.0
                },
                avg_duration_ms: if stats.count > 0 {
                    stats.total_duration.as_millis() as f64 / stats.count as f64
                } else {
                    0.0
                },
                min_duration_ms: stats.min_duration.as_millis() as f64,
                max_duration_ms: stats.max_duration.as_millis() as f64,
            })
            .collect();
        
        let errors = self.errors.values()
            .map(|stats| ErrorSummary {
                error_type: stats.error_type.clone(),
                count: stats.count,
                components: stats.components.clone(),
            })
            .collect();
        
        MetricsSummary {
            uptime_seconds: uptime,
            total_operations: self.total_operations,
            error_count: self.total_errors,
            success_rate,
            avg_response_time,
            operations,
            errors,
        }
    }
    
    /// 重置所有指標
    pub fn reset(&mut self) {
        self.operations.clear();
        self.errors.clear();
        self.start_time = Instant::now();
        self.total_operations = 0;
        self.total_errors = 0;
    }
    
    /// 獲取特定操作的統計
    pub fn get_operation_stats(&self, operation: &str) -> Option<&OperationStats> {
        self.operations.get(operation)
    }
    
    /// 獲取特定錯誤的統計
    pub fn get_error_stats(&self, error_type: &str) -> Option<&ErrorStats> {
        self.errors.get(error_type)
    }
    
    /// 獲取系統運行時間
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// 獲取操作總數
    pub fn get_total_operations(&self) -> u64 {
        self.total_operations
    }
    
    /// 獲取錯誤總數
    pub fn get_total_errors(&self) -> u64 {
        self.total_errors
    }
    
    /// 獲取成功率
    pub fn get_success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            let success_count: u64 = self.operations.values()
                .map(|stats| stats.success_count)
                .sum();
            success_count as f64 / self.total_operations as f64
        } else {
            1.0
        }
    }
}

/// 指標宏，用於簡化指標記錄
#[macro_export]
macro_rules! record_operation {
    ($collector:expr, $operation:expr, $code:block) => {
        {
            let start = std::time::Instant::now();
            let result = $code;
            let duration = start.elapsed();
            let success = result.is_ok();
            
            $collector.record_operation($operation, duration, success).await;
            result
        }
    };
}

/// 企業級效能基準
pub struct PerformanceBenchmarks;

impl PerformanceBenchmarks {
    /// 企業級效能閾值
    pub const MAX_RESPONSE_TIME_MS: f64 = 200.0;
    pub const MIN_SUCCESS_RATE: f64 = 0.99;
    pub const MAX_ERROR_RATE: f64 = 0.01;
    
    /// 檢查是否符合企業級標準
    pub fn meets_enterprise_standards(summary: &MetricsSummary) -> bool {
        summary.avg_response_time <= Self::MAX_RESPONSE_TIME_MS &&
        summary.success_rate >= Self::MIN_SUCCESS_RATE &&
        (summary.error_count as f64 / summary.total_operations as f64) <= Self::MAX_ERROR_RATE
    }
    
    /// 生成效能評估報告
    pub fn generate_assessment(summary: &MetricsSummary) -> PerformanceAssessment {
        PerformanceAssessment {
            meets_standards: Self::meets_enterprise_standards(summary),
            response_time_grade: Self::grade_response_time(summary.avg_response_time),
            success_rate_grade: Self::grade_success_rate(summary.success_rate),
            error_rate_grade: Self::grade_error_rate(
                summary.error_count as f64 / summary.total_operations as f64
            ),
            recommendations: Self::generate_recommendations(summary),
        }
    }
    
    fn grade_response_time(avg_ms: f64) -> Grade {
        match avg_ms {
            t if t <= 50.0 => Grade::Excellent,
            t if t <= 100.0 => Grade::Good,
            t if t <= 200.0 => Grade::Acceptable,
            t if t <= 500.0 => Grade::Poor,
            _ => Grade::Critical,
        }
    }
    
    fn grade_success_rate(rate: f64) -> Grade {
        match rate {
            r if r >= 0.999 => Grade::Excellent,
            r if r >= 0.99 => Grade::Good,
            r if r >= 0.95 => Grade::Acceptable,
            r if r >= 0.90 => Grade::Poor,
            _ => Grade::Critical,
        }
    }
    
    fn grade_error_rate(rate: f64) -> Grade {
        match rate {
            r if r <= 0.001 => Grade::Excellent,
            r if r <= 0.01 => Grade::Good,
            r if r <= 0.05 => Grade::Acceptable,
            r if r <= 0.10 => Grade::Poor,
            _ => Grade::Critical,
        }
    }
    
    fn generate_recommendations(summary: &MetricsSummary) -> Vec<String> {
        let mut recommendations = vec![];
        
        if summary.avg_response_time > Self::MAX_RESPONSE_TIME_MS {
            recommendations.push("優化系統響應時間，考慮快取或非同步處理".to_string());
        }
        
        if summary.success_rate < Self::MIN_SUCCESS_RATE {
            recommendations.push("提升系統可靠性，加強錯誤處理機制".to_string());
        }
        
        if summary.error_count > 0 {
            recommendations.push("分析錯誤模式，實施預防性維護".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("系統效能表現優秀，維持當前標準".to_string());
        }
        
        recommendations
    }
}

/// 效能評估結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAssessment {
    pub meets_standards: bool,
    pub response_time_grade: Grade,
    pub success_rate_grade: Grade,
    pub error_rate_grade: Grade,
    pub recommendations: Vec<String>,
}

/// 評分等級
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Grade {
    Excellent,  // 優秀
    Good,       // 良好
    Acceptable, // 可接受
    Poor,       // 較差
    Critical,   // 關鍵問題
}
