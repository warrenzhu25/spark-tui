use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod app;
mod events;
mod models;
mod parser;
mod ui;

use app::App;

#[derive(Parser)]
#[command(name = "spark-tui")]
#[command(about = "Terminal UI for Apache Spark event log analysis")]
struct Args {
    /// Path to the Spark event log file
    #[arg(short, long)]
    log_file: PathBuf,
    
    /// Test mode - just parse and print summary without starting TUI
    #[arg(short, long)]
    test: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Parse the event log
    let event_log = parser::parse_event_log(&args.log_file)?;
    
    if args.test {
        // Test mode - print summary
        println!("Successfully parsed event log: {}", args.log_file.display());
        println!("Application: {} ({})", event_log.application_info.app_name, event_log.application_info.app_id);
        println!("User: {}, Spark Version: {}", event_log.application_info.user, event_log.application_info.spark_version);
        println!("Jobs: {}", event_log.jobs.len());
        println!("Stages: {}", event_log.stages.len());
        println!("Tasks: {}", event_log.tasks.len());
        println!("Executors: {}", event_log.executors.len());
        println!("SQL Executions: {}", event_log.sql_executions.len());
        println!("Spark Properties: {}", event_log.environment.spark_properties.len());
        return Ok(());
    }
    
    // Create and run the TUI app
    let mut app = App::new(event_log);
    app.run()?;
    
    Ok(())
}