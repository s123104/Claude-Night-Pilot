// 提示模型 - 參考 vibe-kanban 設計

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

/// 提示模板
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Prompt {
    /// 提示 ID
    pub id: String,

    /// 提示名稱
    pub name: String,

    /// 提示內容
    pub content: String,

    /// 提示描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,

    /// 提示類別
    #[serde(default)]
    pub category: PromptCategory,

    /// 標籤
    #[serde(default)]
    pub tags: Vec<String>,

    /// 變數定義
    #[serde(default)]
    pub variables: Vec<PromptVariable>,

    /// 檔案引用 (@ 符號)
    #[serde(default)]
    pub file_references: Vec<FileReference>,

    /// 版本資訊
    pub version: String,

    /// 是否啟用
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// 使用次數
    #[serde(default)]
    pub usage_count: u64,

    /// 平均執行時間 (毫秒)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_execution_time_ms: Option<u64>,

    /// 成功率 (0.0-1.0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub success_rate: Option<f64>,

    /// 最後使用時間
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "string | null")]
    pub last_used_at: Option<DateTime<Utc>>,

    /// 元數據
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// 創建時間
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,

    /// 更新時間
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,

    /// 創建者
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub created_by: Option<String>,
}

/// 提示類別
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub enum PromptCategory {
    #[default]
    General,
    Development,
    Analysis,
    Documentation,
    Testing,
    Automation,
    Research,
    Creative,
    Custom(String),
}

/// 提示變數定義
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PromptVariable {
    /// 變數名稱
    pub name: String,

    /// 變數類型
    pub var_type: VariableType,

    /// 變數描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,

    /// 預設值
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub default_value: Option<String>,

    /// 是否必填
    #[serde(default)]
    pub required: bool,

    /// 驗證規則
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub validation: Option<ValidationRule>,

    /// 可選值列表
    #[serde(default)]
    pub options: Vec<String>,
}

/// 變數類型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum VariableType {
    Text,
    Number,
    Boolean,
    Date,
    Email,
    Url,
    File,
    Directory,
    Choice,
    MultiChoice,
}

/// 驗證規則
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct ValidationRule {
    /// 最小長度
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_length: Option<usize>,

    /// 最大長度
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_length: Option<usize>,

    /// 正則表達式
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub pattern: Option<String>,

    /// 最小值 (數字類型)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub min_value: Option<f64>,

    /// 最大值 (數字類型)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_value: Option<f64>,

    /// 自定義驗證函數
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub custom_validator: Option<String>,
}

/// 檔案引用
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileReference {
    /// 引用名稱 (在提示中的佔位符)
    pub name: String,

    /// 檔案路徑或 glob 模式
    pub path: String,

    /// 引用類型
    pub ref_type: FileReferenceType,

    /// 是否必須存在
    #[serde(default = "default_true")]
    pub required: bool,

    /// 讀取限制 (字節)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_size_bytes: Option<u64>,

    /// 檔案類型限制
    #[serde(default)]
    pub allowed_extensions: Vec<String>,

    /// 描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
}

/// 檔案引用類型
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FileReferenceType {
    /// 單個檔案
    File,
    /// 目錄
    Directory,
    /// Glob 模式
    Glob,
    /// Git 倉庫
    GitRepo,
}

/// 提示範本
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PromptTemplate {
    /// 範本 ID
    pub id: String,

    /// 範本名稱
    pub name: String,

    /// 範本內容
    pub template: String,

    /// 支援的變數
    pub supported_variables: Vec<String>,

    /// 範本描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,

    /// 範本類別
    pub category: PromptCategory,

    /// 是否為系統範本
    #[serde(default)]
    pub is_system_template: bool,
}

impl Prompt {
    /// 創建新提示
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            content: content.into(),
            description: None,
            category: PromptCategory::default(),
            tags: vec![],
            variables: vec![],
            file_references: vec![],
            version: "1.0.0".to_string(),
            enabled: true,
            usage_count: 0,
            avg_execution_time_ms: None,
            success_rate: None,
            last_used_at: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by: None,
        }
    }

    /// 設置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Utc::now();
        self
    }

    /// 設置類別
    pub fn with_category(mut self, category: PromptCategory) -> Self {
        self.category = category;
        self.updated_at = Utc::now();
        self
    }

    /// 添加標籤
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// 添加變數
    pub fn add_variable(&mut self, variable: PromptVariable) {
        // 檢查是否已存在同名變數
        if let Some(existing) = self.variables.iter_mut().find(|v| v.name == variable.name) {
            *existing = variable;
        } else {
            self.variables.push(variable);
        }
        self.updated_at = Utc::now();
    }

    /// 添加檔案引用
    pub fn add_file_reference(&mut self, file_ref: FileReference) {
        // 檢查是否已存在同名引用
        if let Some(existing) = self
            .file_references
            .iter_mut()
            .find(|f| f.name == file_ref.name)
        {
            *existing = file_ref;
        } else {
            self.file_references.push(file_ref);
        }
        self.updated_at = Utc::now();
    }

    /// 渲染提示 (替換變數)
    pub fn render(&self, variables: &HashMap<String, String>) -> Result<String, String> {
        let mut rendered = self.content.clone();

        // 替換變數
        for variable in &self.variables {
            let placeholder = format!("{{{{{}}}}}", variable.name);

            if let Some(value) = variables.get(&variable.name) {
                // 驗證變數值
                if let Some(validation) = &variable.validation {
                    self.validate_variable_value(&variable.name, value, validation)?;
                }
                rendered = rendered.replace(&placeholder, value);
            } else if variable.required {
                return Err(format!("必填變數 '{}' 未提供", variable.name));
            } else if let Some(default) = &variable.default_value {
                rendered = rendered.replace(&placeholder, default);
            }
        }

        // 檢查檔案引用
        for file_ref in &self.file_references {
            if file_ref.required {
                // 這裡可以添加檔案存在性檢查
                // 實際實現時需要檔案系統訪問權限
            }
        }

        Ok(rendered)
    }

    /// 驗證變數值
    fn validate_variable_value(
        &self,
        name: &str,
        value: &str,
        validation: &ValidationRule,
    ) -> Result<(), String> {
        // 長度驗證
        if let Some(min_len) = validation.min_length {
            if value.len() < min_len {
                return Err(format!("變數 '{}' 長度不能少於 {} 字符", name, min_len));
            }
        }

        if let Some(max_len) = validation.max_length {
            if value.len() > max_len {
                return Err(format!("變數 '{}' 長度不能超過 {} 字符", name, max_len));
            }
        }

        // 正則表達式驗證
        if let Some(pattern) = &validation.pattern {
            let regex = regex::Regex::new(pattern)
                .map_err(|_| format!("變數 '{}' 的驗證規則無效", name))?;

            if !regex.is_match(value) {
                return Err(format!("變數 '{}' 格式不正確", name));
            }
        }

        // 數值範圍驗證 (如果可以解析為數字)
        if let (Ok(num_value), Some(min_val)) = (value.parse::<f64>(), validation.min_value) {
            if num_value < min_val {
                return Err(format!("變數 '{}' 值不能小於 {}", name, min_val));
            }
        }

        if let (Ok(num_value), Some(max_val)) = (value.parse::<f64>(), validation.max_value) {
            if num_value > max_val {
                return Err(format!("變數 '{}' 值不能大於 {}", name, max_val));
            }
        }

        Ok(())
    }

    /// 解析提示中的檔案引用
    pub fn parse_file_references(&self) -> Vec<String> {
        let mut references = Vec::new();
        let content = &self.content;

        // 使用正則表達式查找 @file 模式
        let re = regex::Regex::new(r"@([^\s\,\!\?\;]+)").unwrap();

        for captures in re.captures_iter(content) {
            if let Some(file_ref) = captures.get(1) {
                references.push(file_ref.as_str().to_string());
            }
        }

        references
    }

    /// 記錄使用情況
    pub fn record_usage(&mut self, execution_time_ms: Option<u64>, success: bool) {
        self.usage_count += 1;
        self.last_used_at = Some(Utc::now());

        // 更新平均執行時間
        if let Some(exec_time) = execution_time_ms {
            self.avg_execution_time_ms = Some(if let Some(avg) = self.avg_execution_time_ms {
                (avg * (self.usage_count - 1) + exec_time) / self.usage_count
            } else {
                exec_time
            });
        }

        // 更新成功率
        let current_success_rate = self.success_rate.unwrap_or(1.0);
        let success_value = if success { 1.0 } else { 0.0 };

        self.success_rate = Some(
            (current_success_rate * (self.usage_count - 1) as f64 + success_value)
                / self.usage_count as f64,
        );

        self.updated_at = Utc::now();
    }

    /// 檢查是否包含檔案引用
    pub fn has_file_references(&self) -> bool {
        !self.file_references.is_empty() || self.content.contains('@')
    }

    /// 獲取必填變數
    pub fn required_variables(&self) -> Vec<&PromptVariable> {
        self.variables.iter().filter(|v| v.required).collect()
    }

    /// 驗證提示完整性
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("提示名稱不能為空".to_string());
        }

        if self.content.trim().is_empty() {
            errors.push("提示內容不能為空".to_string());
        }

        // 檢查變數定義與內容是否匹配
        for variable in &self.variables {
            let placeholder = format!("{{{{{}}}}}", variable.name);
            if !self.content.contains(&placeholder) {
                errors.push(format!("定義的變數 '{}' 在內容中未使用", variable.name));
            }
        }

        // 檢查檔案引用定義
        for file_ref in &self.file_references {
            let ref_pattern = format!("@{}", file_ref.name);
            if !self.content.contains(&ref_pattern) {
                errors.push(format!("定義的檔案引用 '{}' 在內容中未使用", file_ref.name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl PromptVariable {
    /// 創建新變數
    pub fn new(name: impl Into<String>, var_type: VariableType) -> Self {
        Self {
            name: name.into(),
            var_type,
            description: None,
            default_value: None,
            required: false,
            validation: None,
            options: vec![],
        }
    }

    /// 設置為必填
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 設置預設值
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// 設置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 設置驗證規則
    pub fn with_validation(mut self, validation: ValidationRule) -> Self {
        self.validation = Some(validation);
        self
    }

    /// 設置選項 (用於 Choice 和 MultiChoice 類型)
    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }
}

fn default_true() -> bool {
    true
}

impl std::fmt::Display for PromptCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptCategory::General => write!(f, "一般"),
            PromptCategory::Development => write!(f, "開發"),
            PromptCategory::Analysis => write!(f, "分析"),
            PromptCategory::Documentation => write!(f, "文檔"),
            PromptCategory::Testing => write!(f, "測試"),
            PromptCategory::Automation => write!(f, "自動化"),
            PromptCategory::Research => write!(f, "研究"),
            PromptCategory::Creative => write!(f, "創意"),
            PromptCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableType::Text => write!(f, "文本"),
            VariableType::Number => write!(f, "數字"),
            VariableType::Boolean => write!(f, "布林"),
            VariableType::Date => write!(f, "日期"),
            VariableType::Email => write!(f, "郵箱"),
            VariableType::Url => write!(f, "網址"),
            VariableType::File => write!(f, "檔案"),
            VariableType::Directory => write!(f, "目錄"),
            VariableType::Choice => write!(f, "單選"),
            VariableType::MultiChoice => write!(f, "多選"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new("測試提示", "Hello {{name}}!");

        assert!(!prompt.id.is_empty());
        assert_eq!(prompt.name, "測試提示");
        assert_eq!(prompt.content, "Hello {{name}}!");
        assert!(prompt.enabled);
    }

    #[test]
    fn test_prompt_variable_rendering() {
        let mut prompt = Prompt::new("變數測試", "Hello {{name}}, you are {{age}} years old!");

        prompt.add_variable(PromptVariable::new("name", VariableType::Text).required());
        prompt.add_variable(PromptVariable::new("age", VariableType::Number).with_default("25"));

        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "Alice".to_string());

        let rendered = prompt.render(&variables).unwrap();
        assert_eq!(rendered, "Hello Alice, you are 25 years old!");
    }

    #[test]
    fn test_prompt_validation() {
        let mut prompt = Prompt::new("驗證測試", "Hello {{name}}!");

        prompt.add_variable(
            PromptVariable::new("email", VariableType::Email).with_validation(ValidationRule {
                pattern: Some(r"^[^\s@]+@[^\s@]+\.[^\s@]+$".to_string()),
                ..Default::default()
            }),
        );

        // 這個變數定義了但沒在內容中使用，應該產生驗證錯誤
        let result = prompt.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_file_reference_parsing() {
        let prompt = Prompt::new(
            "檔案引用測試",
            "分析這個檔案: @src/main.rs 和這個目錄: @tests/",
        );

        let references = prompt.parse_file_references();
        assert_eq!(references, vec!["src/main.rs", "tests/"]);
    }

    #[test]
    fn test_usage_recording() {
        let mut prompt = Prompt::new("使用統計測試", "Simple prompt");

        // 記錄第一次使用
        prompt.record_usage(Some(1000), true);
        assert_eq!(prompt.usage_count, 1);
        assert_eq!(prompt.avg_execution_time_ms, Some(1000));
        assert_eq!(prompt.success_rate, Some(1.0));

        // 記錄第二次使用 (失敗)
        prompt.record_usage(Some(1500), false);
        assert_eq!(prompt.usage_count, 2);
        assert_eq!(prompt.avg_execution_time_ms, Some(1250)); // (1000+1500)/2
        assert_eq!(prompt.success_rate, Some(0.5)); // 1成功1失敗
    }
}
