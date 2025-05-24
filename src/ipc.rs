use anyhow::Result;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub message_type: MessageType,
    pub timeout: Option<u64>, // 超时时间（秒）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Error,
}

impl Message {
    pub fn new_request(content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            message_type: MessageType::Request,
            timeout: None,
        }
    }

    pub fn new_request_with_timeout(content: String, timeout: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            message_type: MessageType::Request,
            timeout: Some(timeout),
        }
    }

    pub fn new_response(id: String, content: String) -> Self {
        Self {
            id,
            content,
            message_type: MessageType::Response,
            timeout: None,
        }
    }

    pub fn new_error(id: String, content: String) -> Self {
        Self {
            id,
            content,
            message_type: MessageType::Error,
            timeout: None,
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    let mut path = dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap_or_else(|| std::env::temp_dir());
    path.push("ai-review.sock");
    path
}

pub struct IpcServer {
    listener: LocalSocketListener,
    message_sender: tokio_mpsc::UnboundedSender<(Message, mpsc::Sender<String>)>,
}

impl IpcServer {
    pub fn new(message_sender: tokio_mpsc::UnboundedSender<(Message, mpsc::Sender<String>)>) -> Result<Self> {
        let socket_path = get_socket_path();

        // 删除可能存在的旧socket文件
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = LocalSocketListener::bind(socket_path)?;

        Ok(Self {
            listener,
            message_sender,
        })
    }

    pub async fn run(&self) -> Result<()> {
        println!("🔧 IPC服务器开始监听连接...");
        loop {
            match self.listener.accept() {
                Ok(stream) => {
                    println!("🔗 新客户端已连接");
                    let sender = self.message_sender.clone();

                    // 暂时使用同步处理来调试
                    println!("🚀 开始处理客户端");
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = handle_client_with_timeout(stream, sender) {
                            eprintln!("❌ 处理客户端时出错: {}", e);
                        } else {
                            println!("✅ 客户端处理成功");
                        }
                    });
                }
                Err(e) => {
                    eprintln!("❌ 接受连接时出错: {}", e);
                }
            }
        }
    }
}

fn handle_client_with_timeout(
    mut stream: LocalSocketStream,
    message_sender: tokio_mpsc::UnboundedSender<(Message, mpsc::Sender<String>)>,
) -> Result<()> {
    println!("🔧 处理客户端连接");
    // 读取消息
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer)?;
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("🔍 收到 {} 字节数据: {}", bytes_read, received_data);

    // 查找第一个换行符
    if let Some(newline_pos) = received_data.find('\n') {
        let line = &received_data[..newline_pos];
        println!("🔍 解析行: {}", line);
        let message: Message = serde_json::from_str(line.trim())?;
        println!("🔍 解析消息: {:?}", message);

        // 创建响应通道
        let (response_tx, response_rx) = mpsc::channel();

        // 发送消息到UI
        println!("🔍 发送消息到UI处理程序");
        if let Err(e) = message_sender.send((message.clone(), response_tx)) {
            eprintln!("❌ 发送消息到UI处理程序失败: {}", e);
            return Err(anyhow::anyhow!("❌ 发送消息到UI处理程序失败"));
        }
        println!("✅ 消息已成功发送给UI处理程序");

        // 使用消息中指定的超时时间，默认30秒
        let timeout_secs = message.timeout.unwrap_or(30);
        
        // 等待UI响应 (使用自定义超时)
        match response_rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(user_response) => {
                let response = Message::new_response(message.id, user_response);
                let response_json = serde_json::to_string(&response)?;
                writeln!(stream, "{}", response_json)?;
            }
            Err(_) => {
                let error = Message::new_error(message.id, format!("❌ 超时未收到回复 ({}秒)", timeout_secs));
                let error_json = serde_json::to_string(&error)?;
                writeln!(stream, "{}", error_json)?;
            }
        }
    }

    Ok(())
}

async fn handle_client_with_timeout_async(
    mut stream: LocalSocketStream,
    message_sender: tokio_mpsc::UnboundedSender<(Message, mpsc::Sender<String>)>,
) -> Result<()> {
    println!("🔄 正在处理客户端连接");
    
    // 读取消息 - 使用阻塞IO但在异步任务中运行
    let mut buffer = [0; 4096];
    let read_result = tokio::task::spawn_blocking(move || {
        stream.read(&mut buffer).map(|n| (n, buffer, stream))
    }).await??;
    
    let (bytes_read, buffer, mut stream) = read_result;
    let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("📨 接收到 {} 字节数据: {}", bytes_read, received_data);

    // 查找第一个换行符
    if let Some(newline_pos) = received_data.find('\n') {
        let line = &received_data[..newline_pos];
        println!("🔍 解析行: {}", line);
        let message: Message = serde_json::from_str(line.trim())?;
        println!("✅ 解析消息成功: {:?}", message);

        // 创建响应通道
        let (response_tx, response_rx) = mpsc::channel();

        // 发送消息到UI
        println!("📤 正在发送消息到UI处理器");
        if let Err(e) = message_sender.send((message.clone(), response_tx)) {
            eprintln!("❌ 发送消息到UI处理器失败: {}", e);
            return Err(anyhow::anyhow!("Failed to send message to UI handler"));
        }
        println!("✅ 消息已成功发送到UI处理器");

        // 使用消息中指定的超时时间，默认30秒
        let timeout_secs = message.timeout.unwrap_or(30);
        println!("⏰ 等待UI响应，超时时间: {}秒", timeout_secs);
        
        // 等待UI响应 (使用自定义超时)
        match response_rx.recv_timeout(Duration::from_secs(timeout_secs)) {
            Ok(user_response) => {
                println!("✅ 收到用户响应: {}", user_response);
                let response = Message::new_response(message.id, user_response);
                let response_json = serde_json::to_string(&response)?;
                
                // 异步写入响应
                tokio::task::spawn_blocking(move || {
                    writeln!(stream, "{}", response_json)
                }).await??;
                println!("📤 响应已发送回客户端");
            }
            Err(_) => {
                println!("⏰ 等待响应超时");
                let error = Message::new_error(message.id, format!("超时未收到回复 ({}秒)", timeout_secs));
                let error_json = serde_json::to_string(&error)?;
                
                // 异步写入错误响应
                tokio::task::spawn_blocking(move || {
                    writeln!(stream, "{}", error_json)
                }).await??;
                println!("📤 超时错误已发送回客户端");
            }
        }
    }

    Ok(())
}

pub struct IpcClient;

impl IpcClient {
    pub fn send_message(content: String) -> Result<String> {
        Self::send_message_with_timeout(content, 30)
    }

    pub fn send_message_with_timeout(content: String, timeout: u64) -> Result<String> {
        let socket_path = get_socket_path();
        let mut stream = LocalSocketStream::connect(socket_path)?;

        let message = Message::new_request_with_timeout(content, timeout);
        let message_json = serde_json::to_string(&message)?;

        writeln!(stream, "{}", message_json)?;

        // 读取响应
        let mut buffer = [0; 4096];
        let bytes_read = stream.read(&mut buffer)?;
        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);

        // 查找第一个换行符
        if let Some(newline_pos) = received_data.find('\n') {
            let line = &received_data[..newline_pos];
            let response: Message = serde_json::from_str(line.trim())?;
            
            // 检查是否是错误响应
            match response.message_type {
                MessageType::Error => Err(anyhow::anyhow!("{}", response.content)),
                _ => Ok(response.content),
            }
        } else {
            Err(anyhow::anyhow!("❌ 未收到有效响应"))
        }
    }
}
