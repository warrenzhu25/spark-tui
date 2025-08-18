use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::models::*;

pub fn parse_event_log(log_path: &Path) -> Result<SparkEventLog> {
    let file = File::open(log_path)
        .with_context(|| format!("Failed to open event log: {}", log_path.display()))?;
    
    let reader = BufReader::new(file);
    
    let mut application_info = None;
    let mut jobs = HashMap::new();
    let mut stages = HashMap::new();
    let mut tasks = HashMap::new();
    let mut executors = HashMap::new();
    let mut environment = Environment {
        spark_properties: HashMap::new(),
        hadoop_properties: HashMap::new(),
        system_properties: HashMap::new(),
        classpath_entries: HashMap::new(),
    };
    
    for line in reader.lines() {
        let line = line.context("Failed to read line from event log")?;
        let event: Value = serde_json::from_str(&line)
            .context("Failed to parse JSON from event log line")?;
        
        if let Some(event_type) = event.get("Event").and_then(|v| v.as_str()) {
            match event_type {
                "SparkListenerApplicationStart" => {
                    application_info = Some(parse_application_start(&event)?);
                }
                "SparkListenerApplicationEnd" => {
                    if let Some(ref mut app_info) = application_info {
                        app_info.end_time = parse_timestamp(&event, "Timestamp");
                    }
                }
                "SparkListenerJobStart" => {
                    let job = parse_job_start(&event)?;
                    jobs.insert(job.job_id, job);
                }
                "SparkListenerJobEnd" => {
                    if let Some(job_id) = event.get("Job ID").and_then(|v| v.as_u64()) {
                        if let Some(job) = jobs.get_mut(&job_id) {
                            job.completion_time = parse_timestamp(&event, "Completion Time");
                            job.status = parse_job_result(&event);
                        }
                    }
                }
                "SparkListenerStageSubmitted" => {
                    let stage = parse_stage_submitted(&event)?;
                    stages.insert(stage.stage_id, stage);
                }
                "SparkListenerStageCompleted" => {
                    if let Some(stage_info) = event.get("Stage Info") {
                        if let Some(stage_id) = stage_info.get("Stage ID").and_then(|v| v.as_u64()) {
                            if let Some(stage) = stages.get_mut(&stage_id) {
                                stage.completion_time = parse_timestamp(stage_info, "Completion Time");
                                stage.status = if stage_info.get("Failure Reason").is_some() {
                                    StageStatus::Failed
                                } else {
                                    StageStatus::Complete
                                };
                            }
                        }
                    }
                }
                "SparkListenerTaskStart" => {
                    let task = parse_task_start(&event)?;
                    tasks.insert(task.task_id, task);
                }
                "SparkListenerTaskEnd" => {
                    if let Some(task_info) = event.get("Task Info") {
                        if let Some(task_id) = task_info.get("Task ID").and_then(|v| v.as_u64()) {
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.finish_time = parse_timestamp(task_info, "Finish Time");
                                task.status = parse_task_status(task_info);
                                task.metrics = parse_task_metrics(&event);
                            }
                        }
                    }
                }
                "SparkListenerExecutorAdded" => {
                    let executor = parse_executor_added(&event)?;
                    executors.insert(executor.executor_id.clone(), executor);
                }
                "SparkListenerExecutorRemoved" => {
                    if let Some(executor_id) = event.get("Executor ID").and_then(|v| v.as_str()) {
                        if let Some(executor) = executors.get_mut(executor_id) {
                            executor.is_active = false;
                        }
                    }
                }
                "SparkListenerEnvironmentUpdate" => {
                    environment = parse_environment_update(&event)?;
                }
                _ => {
                    // Ignore other event types for now
                }
            }
        }
    }
    
    let application_info = application_info
        .context("No application start event found in event log")?;
    
    Ok(SparkEventLog {
        application_info,
        jobs,
        stages,
        tasks,
        executors,
        environment,
    })
}

fn parse_application_start(event: &Value) -> Result<ApplicationInfo> {
    let app_name = event.get("App Name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let app_id = event.get("App ID")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let start_time = parse_timestamp(event, "Timestamp")
        .unwrap_or_else(|| Utc::now());
    
    let user = event.get("User")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let spark_version = event.get("Spark Version")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    Ok(ApplicationInfo {
        app_id,
        app_name,
        app_attempt_id: event.get("App Attempt ID").and_then(|v| v.as_str()).map(|s| s.to_string()),
        start_time,
        end_time: None,
        user,
        spark_version,
    })
}

fn parse_job_start(event: &Value) -> Result<Job> {
    let job_id = event.get("Job ID")
        .and_then(|v| v.as_u64())
        .context("Missing Job ID")?;
    
    let submission_time = parse_timestamp(event, "Submission Time")
        .unwrap_or_else(|| Utc::now());
    
    let stage_ids = event.get("Stage IDs")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
        .unwrap_or_default();
    
    Ok(Job {
        job_id,
        name: format!("Job {}", job_id),
        description: None,
        submission_time,
        completion_time: None,
        stage_ids,
        status: JobStatus::Running,
        num_tasks: 0,
        num_active_tasks: 0,
        num_completed_tasks: 0,
        num_skipped_tasks: 0,
        num_failed_tasks: 0,
    })
}

fn parse_stage_submitted(event: &Value) -> Result<Stage> {
    let stage_info = event.get("Stage Info")
        .context("Missing Stage Info")?;
    
    let stage_id = stage_info.get("Stage ID")
        .and_then(|v| v.as_u64())
        .context("Missing Stage ID")?;
    
    let stage_attempt_id = stage_info.get("Stage Attempt ID")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let name = stage_info.get("Stage Name")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("Stage {}", stage_id))
        .to_string();
    
    let num_tasks = stage_info.get("Number of Tasks")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let submission_time = parse_timestamp(stage_info, "Submission Time");
    
    let parent_ids = stage_info.get("Parent IDs")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
        .unwrap_or_default();
    
    let rdd_info = stage_info.get("RDD Info")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| parse_rdd_info(v).ok()).collect())
        .unwrap_or_default();
    
    Ok(Stage {
        stage_id,
        stage_attempt_id,
        name,
        num_tasks,
        parent_ids,
        rdd_info,
        submission_time,
        completion_time: None,
        status: StageStatus::Active,
        task_metrics: None,
    })
}

fn parse_task_start(event: &Value) -> Result<Task> {
    let task_info = event.get("Task Info")
        .context("Missing Task Info")?;
    
    let task_id = task_info.get("Task ID")
        .and_then(|v| v.as_u64())
        .context("Missing Task ID")?;
    
    let stage_id = task_info.get("Stage ID")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let stage_attempt_id = task_info.get("Stage Attempt ID")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let partition_id = task_info.get("Partition ID")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let executor_id = task_info.get("Executor ID")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let host = task_info.get("Host")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let launch_time = parse_timestamp(task_info, "Launch Time")
        .unwrap_or_else(|| Utc::now());
    
    Ok(Task {
        task_id,
        stage_id,
        stage_attempt_id,
        partition_id,
        executor_id,
        host,
        launch_time,
        finish_time: None,
        status: TaskStatus::Running,
        metrics: None,
    })
}

fn parse_executor_added(event: &Value) -> Result<Executor> {
    let executor_id = event.get("Executor ID")
        .and_then(|v| v.as_str())
        .context("Missing Executor ID")?
        .to_string();
    
    let executor_info = event.get("Executor Info")
        .context("Missing Executor Info")?;
    
    let host = executor_info.get("Host")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let total_cores = executor_info.get("Total Cores")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;
    
    let max_memory = executor_info.get("Maximum Memory")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    Ok(Executor {
        executor_id,
        host,
        port: 0,
        is_active: true,
        total_cores,
        max_tasks: total_cores,
        active_tasks: 0,
        failed_tasks: 0,
        completed_tasks: 0,
        total_tasks: 0,
        total_duration: 0,
        total_gc_time: 0,
        total_input_bytes: 0,
        total_shuffle_read: 0,
        total_shuffle_write: 0,
        max_memory,
        max_on_heap_memory: 0,
        max_off_heap_memory: 0,
        memory_used: 0,
        disk_used: 0,
    })
}

fn parse_environment_update(event: &Value) -> Result<Environment> {
    let spark_properties = parse_properties(event, "Spark Properties");
    let hadoop_properties = parse_properties(event, "Hadoop Properties");
    let system_properties = parse_properties(event, "System Properties");
    let classpath_entries = parse_properties(event, "Classpath Entries");
    
    Ok(Environment {
        spark_properties,
        hadoop_properties,
        system_properties,
        classpath_entries,
    })
}

fn parse_properties(event: &Value, key: &str) -> HashMap<String, String> {
    event.get(key)
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_rdd_info(value: &Value) -> Result<RddInfo> {
    let rdd_id = value.get("RDD ID")
        .and_then(|v| v.as_u64())
        .context("Missing RDD ID")?;
    
    let name = value.get("Name")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("RDD {}", rdd_id))
        .to_string();
    
    Ok(RddInfo {
        rdd_id,
        name,
        num_partitions: value.get("Number of Partitions").and_then(|v| v.as_u64()).unwrap_or(0),
        storage_level: value.get("Storage Level").and_then(|v| v.as_str()).unwrap_or("NONE").to_string(),
        num_cached_partitions: value.get("Number of Cached Partitions").and_then(|v| v.as_u64()).unwrap_or(0),
        memory_size: value.get("Memory Size").and_then(|v| v.as_u64()).unwrap_or(0),
        disk_size: value.get("Disk Size").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_task_metrics(event: &Value) -> Option<TaskMetrics> {
    let metrics = event.get("Task Metrics")?;
    
    Some(TaskMetrics {
        execution_time: metrics.get("Executor Run Time").and_then(|v| v.as_u64()).unwrap_or(0),
        cpu_time: metrics.get("Executor CPU Time").and_then(|v| v.as_u64()).unwrap_or(0),
        gc_time: metrics.get("JVM GC Time").and_then(|v| v.as_u64()).unwrap_or(0),
        result_size: metrics.get("Result Size").and_then(|v| v.as_u64()).unwrap_or(0),
        jvm_gc_time: metrics.get("JVM GC Time").and_then(|v| v.as_u64()).unwrap_or(0),
        result_serialization_time: metrics.get("Result Serialization Time").and_then(|v| v.as_u64()).unwrap_or(0),
        memory_bytes_spilled: metrics.get("Memory Bytes Spilled").and_then(|v| v.as_u64()).unwrap_or(0),
        disk_bytes_spilled: metrics.get("Disk Bytes Spilled").and_then(|v| v.as_u64()).unwrap_or(0),
        peak_execution_memory: metrics.get("Peak Execution Memory").and_then(|v| v.as_u64()).unwrap_or(0),
        input_metrics: parse_input_metrics(metrics),
        output_metrics: parse_output_metrics(metrics),
        shuffle_read_metrics: parse_shuffle_read_metrics(metrics),
        shuffle_write_metrics: parse_shuffle_write_metrics(metrics),
    })
}

fn parse_input_metrics(metrics: &Value) -> Option<InputMetrics> {
    let input = metrics.get("Input Metrics")?;
    Some(InputMetrics {
        bytes_read: input.get("Bytes Read").and_then(|v| v.as_u64()).unwrap_or(0),
        records_read: input.get("Records Read").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_output_metrics(metrics: &Value) -> Option<OutputMetrics> {
    let output = metrics.get("Output Metrics")?;
    Some(OutputMetrics {
        bytes_written: output.get("Bytes Written").and_then(|v| v.as_u64()).unwrap_or(0),
        records_written: output.get("Records Written").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_shuffle_read_metrics(metrics: &Value) -> Option<ShuffleReadMetrics> {
    let shuffle_read = metrics.get("Shuffle Read Metrics")?;
    Some(ShuffleReadMetrics {
        remote_blocks_fetched: shuffle_read.get("Remote Blocks Fetched").and_then(|v| v.as_u64()).unwrap_or(0),
        local_blocks_fetched: shuffle_read.get("Local Blocks Fetched").and_then(|v| v.as_u64()).unwrap_or(0),
        fetch_wait_time: shuffle_read.get("Fetch Wait Time").and_then(|v| v.as_u64()).unwrap_or(0),
        remote_bytes_read: shuffle_read.get("Remote Bytes Read").and_then(|v| v.as_u64()).unwrap_or(0),
        local_bytes_read: shuffle_read.get("Local Bytes Read").and_then(|v| v.as_u64()).unwrap_or(0),
        records_read: shuffle_read.get("Records Read").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_shuffle_write_metrics(metrics: &Value) -> Option<ShuffleWriteMetrics> {
    let shuffle_write = metrics.get("Shuffle Write Metrics")?;
    Some(ShuffleWriteMetrics {
        bytes_written: shuffle_write.get("Bytes Written").and_then(|v| v.as_u64()).unwrap_or(0),
        write_time: shuffle_write.get("Write Time").and_then(|v| v.as_u64()).unwrap_or(0),
        records_written: shuffle_write.get("Records Written").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

fn parse_timestamp(event: &Value, key: &str) -> Option<DateTime<Utc>> {
    event.get(key)
        .and_then(|v| v.as_u64())
        .and_then(|timestamp| {
            DateTime::from_timestamp(timestamp as i64 / 1000, ((timestamp % 1000) * 1_000_000) as u32)
        })
}

fn parse_job_result(event: &Value) -> JobStatus {
    match event.get("Job Result").and_then(|v| v.get("Result")).and_then(|v| v.as_str()) {
        Some("JobSucceeded") => JobStatus::Succeeded,
        Some("JobFailed") => JobStatus::Failed,
        _ => JobStatus::Unknown,
    }
}

fn parse_task_status(task_info: &Value) -> TaskStatus {
    if task_info.get("Failed").and_then(|v| v.as_bool()).unwrap_or(false) {
        TaskStatus::Failed
    } else if task_info.get("Killed").and_then(|v| v.as_bool()).unwrap_or(false) {
        TaskStatus::Killed
    } else if task_info.get("Finished").and_then(|v| v.as_bool()).unwrap_or(false) {
        TaskStatus::Success
    } else {
        TaskStatus::Running
    }
}