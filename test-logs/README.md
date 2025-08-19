# Test Event Logs

This directory contains realistic Spark event logs that demonstrate common performance issues and failure scenarios. These logs are designed to test the capabilities of the Spark TUI application.

## Available Test Scenarios

### 1. OOM (Out of Memory) Scenario - `oom-scenario.json`

**Problem**: Memory pressure leading to OutOfMemoryError and executor failures.

**Key Issues Demonstrated**:
- Java heap space exhaustion during shuffle operations
- GC overhead limit exceeded during spill operations
- High memory spillage to disk (2GB+ spilled)
- Tasks failing with OutOfMemoryError
- Executor loss due to memory issues
- Job failure due to repeated task failures

**Metrics to Observe**:
- Memory Bytes Spilled: 524MB - 3.2GB per task
- JVM GC Time: 1.2s - 4.8s (very high)
- Peak Execution Memory: 805MB - 4.3GB
- Failed tasks with OutOfMemoryError stack traces
- Executor removal due to command exit code 1

**Configuration Issues**:
- `spark.executor.memory`: Only 1g (too low for workload)
- `spark.sql.adaptive.enabled`: false (no optimization)
- Large dataset with 200 tasks on limited memory

---

### 2. Data Skew Scenario - `data-skew-scenario.json`

**Problem**: Uneven data distribution causing some tasks to process significantly more data.

**Key Issues Demonstrated**:
- Extreme variation in task execution times (850ms vs 5.8s)
- Unbalanced shuffle read sizes (5KB vs 1GB)
- Memory spillage on skewed partitions (3.2GB spilled)
- Remote shuffle fetches causing network bottlenecks
- One partition processing 10x more records than others

**Metrics to Observe**:
- Task 0: 1GB input, 10M records, 5.8s execution
- Task 9: 524MB input, 5M records, 5.15s execution  
- Task 1-8: 1-20MB input, 1K-42K records, <1.5s execution
- Shuffle Read: 500KB-850MB per task (1000x difference)
- Memory spilled: 0 for most tasks, 3.2GB for skewed tasks

**Adaptive Features Enabled**:
- `spark.sql.adaptive.enabled`: true
- `spark.sql.adaptive.skewJoin.enabled`: true
- `spark.sql.adaptive.skewJoin.skewedPartitionThresholdInBytes`: 256MB

---

### 3. Slow Tasks Scenario - `slow-tasks-scenario.json`

**Problem**: Hardware heterogeneity causing significant task runtime variation.

**Key Issues Demonstrated**:
- Fast workers (1.6-1.8s per task) vs slow workers (7.7-8.8s per task)
- 5x performance difference between hardware types
- Memory spillage on slower nodes (up to 268MB spilled)
- Higher GC overhead on slower hardware (1s+ GC time)
- Task speculation opportunities

**Metrics to Observe**:
- Fast workers: 1.6-1.85s execution, minimal spillage
- Slow workers: 7.7-8.85s execution, significant spillage (up to 268MB)
- GC Time: 80-105ms (fast) vs 620-1000ms (slow)
- Peak Memory: 134-155MB (fast) vs 402-537MB (slow)
- Same workload, different hardware performance

**Speculation Configuration**:
- `spark.speculation`: true
- `spark.speculation.multiplier`: 2.0
- `spark.speculation.quantile`: 0.75

---

### 4. Executor Failures Scenario - `executor-failures-scenario.json`

**Problem**: Infrastructure instability causing executor failures and dynamic replacement.

**Key Issues Demonstrated**:
- Host decommissioning during job execution
- Executor process crashes (exit code 143)
- Task commit failures and conflicts
- Dynamic allocation replacing failed executors
- Successful fault tolerance and recovery

**Metrics to Observe**:
- Executor 3: Lost due to "Host unreliable-worker1 decommissioned"
- Executor 4: Lost due to "Command exited with code 143"  
- 4 tasks lost on executor 3, 4 tasks lost on executor 4
- Replacement executors 5 and 6 added automatically
- Failed tasks successfully retried on replacement executors
- Job completes successfully despite multiple executor failures

**Fault Tolerance Configuration**:
- `spark.dynamicAllocation.enabled`: true
- `spark.task.maxAttempts`: 3
- `spark.stage.maxConsecutiveAttempts`: 8
- `spark.blacklist.enabled`: true

## Testing Usage

You can test these scenarios with the Spark TUI:

```bash
# Test OOM scenario
cargo run -- --log-file test-logs/oom-scenario.json

# Test data skew scenario  
cargo run -- --log-file test-logs/data-skew-scenario.json

# Test slow tasks scenario
cargo run -- --log-file test-logs/slow-tasks-scenario.json

# Test executor failures scenario
cargo run -- --log-file test-logs/executor-failures-scenario.json

# Quick parse test (no TUI)
cargo run -- --log-file test-logs/oom-scenario.json --test
```

## What to Look For

When analyzing these logs with the TUI:

### Jobs Tab
- Job success/failure status
- Execution duration variations
- Stage and task counts

### Stages Tab  
- Stage completion times
- Task count and failure rates
- Stage-level metrics aggregation

### Executors Tab
- Memory usage patterns
- Task distribution across executors
- Executor failure events and replacements
- GC time variations

### Environment Tab
- Configuration parameters affecting performance
- Memory and core allocations
- Adaptive query execution settings

These scenarios represent real-world Spark performance issues and demonstrate how the TUI can help diagnose and understand application behavior.