use serde::{Deserialize, Serialize};

/// Acemcp搜索请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcemcpRequest {
    /// 项目根目录的绝对路径
    pub project_root_path: String,
    /// 用于查找相关代码上下文的自然语言搜索查询
    pub query: String,
}

/// Acemcp配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcemcpConfig {
    /// API端点URL
    pub base_url: Option<String>,
    /// 认证令牌
    pub token: Option<String>,
    /// 每批上传的文件数量
    pub batch_size: Option<u32>,
    /// 大文件分割前的最大行数
    pub max_lines_per_blob: Option<u32>,
    /// 要索引的文件扩展名列表
    pub text_extensions: Option<Vec<String>>,
    /// 要排除的模式列表
    pub exclude_patterns: Option<Vec<String>>,
}
