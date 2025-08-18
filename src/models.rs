use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SparkEventLog {
    pub application_info: ApplicationInfo,
    pub jobs: HashMap<u64, Job>,
    pub stages: HashMap<u64, Stage>,
    pub tasks: HashMap<u64, Task>,
    pub executors: HashMap<String, Executor>,
    pub environment: Environment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApplicationInfo {
    pub app_id: String,
    pub app_name: String,
    pub app_attempt_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub user: String,
    pub spark_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    pub job_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub submission_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,
    pub stage_ids: Vec<u64>,
    pub status: JobStatus,
    pub num_tasks: u64,
    pub num_active_tasks: u64,
    pub num_completed_tasks: u64,
    pub num_skipped_tasks: u64,
    pub num_failed_tasks: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum JobStatus {
    Running,
    Succeeded,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stage {
    pub stage_id: u64,
    pub stage_attempt_id: u64,
    pub name: String,
    pub num_tasks: u64,
    pub parent_ids: Vec<u64>,
    pub rdd_info: Vec<RddInfo>,
    pub submission_time: Option<DateTime<Utc>>,
    pub completion_time: Option<DateTime<Utc>>,
    pub status: StageStatus,
    pub task_metrics: Option<TaskMetrics>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StageStatus {
    Active,
    Complete,
    Failed,
    Pending,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub task_id: u64,
    pub stage_id: u64,
    pub stage_attempt_id: u64,
    pub partition_id: u64,
    pub executor_id: String,
    pub host: String,
    pub launch_time: DateTime<Utc>,
    pub finish_time: Option<DateTime<Utc>>,
    pub status: TaskStatus,
    pub metrics: Option<TaskMetrics>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TaskStatus {
    Running,
    Success,
    Failed,
    Killed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskMetrics {
    pub execution_time: u64,
    pub cpu_time: u64,
    pub gc_time: u64,
    pub result_size: u64,
    pub jvm_gc_time: u64,
    pub result_serialization_time: u64,
    pub memory_bytes_spilled: u64,
    pub disk_bytes_spilled: u64,
    pub peak_execution_memory: u64,
    pub input_metrics: Option<InputMetrics>,
    pub output_metrics: Option<OutputMetrics>,
    pub shuffle_read_metrics: Option<ShuffleReadMetrics>,
    pub shuffle_write_metrics: Option<ShuffleWriteMetrics>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputMetrics {
    pub bytes_read: u64,
    pub records_read: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputMetrics {
    pub bytes_written: u64,
    pub records_written: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShuffleReadMetrics {
    pub remote_blocks_fetched: u64,
    pub local_blocks_fetched: u64,
    pub fetch_wait_time: u64,
    pub remote_bytes_read: u64,
    pub local_bytes_read: u64,
    pub records_read: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShuffleWriteMetrics {
    pub bytes_written: u64,
    pub write_time: u64,
    pub records_written: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Executor {
    pub executor_id: String,
    pub host: String,
    pub port: u16,
    pub is_active: bool,
    pub total_cores: u32,
    pub max_tasks: u32,
    pub active_tasks: u32,
    pub failed_tasks: u32,
    pub completed_tasks: u32,
    pub total_tasks: u32,
    pub total_duration: u64,
    pub total_gc_time: u64,
    pub total_input_bytes: u64,
    pub total_shuffle_read: u64,
    pub total_shuffle_write: u64,
    pub max_memory: u64,
    pub max_on_heap_memory: u64,
    pub max_off_heap_memory: u64,
    pub memory_used: u64,
    pub disk_used: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RddInfo {
    pub rdd_id: u64,
    pub name: String,
    pub num_partitions: u64,
    pub storage_level: String,
    pub num_cached_partitions: u64,
    pub memory_size: u64,
    pub disk_size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Environment {
    pub spark_properties: HashMap<String, String>,
    pub hadoop_properties: HashMap<String, String>,
    pub system_properties: HashMap<String, String>,
    pub classpath_entries: HashMap<String, String>,
}