use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Running,
    Stopped,
    Pending,
    Failed,
    Unknown(String),
}

impl Status {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "RUNNING" => Status::Running,
            "IDLE" | "STOPPED" | "TERMINATED" | "DELETED" => Status::Stopped,
            "PENDING" | "STARTING" | "RESTARTING" | "DELETING" => Status::Pending,
            "FAILED" | "ERROR" => Status::Failed,
            other => Status::Unknown(other.to_string()),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Status::Running => "RUNNING",
            Status::Stopped => "IDLE",
            Status::Pending => "PENDING",
            Status::Failed => "FAILED",
            Status::Unknown(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Shape {
    List(Vec<ListItem>),
    Table(TableData),
    Badge(BadgeData),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub name: String,
    pub status: Status,
    pub detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct BadgeData {
    pub label: String,
    pub value: String,
}
