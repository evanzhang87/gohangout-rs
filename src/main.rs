//! GoHangout-rs - Main entry point
//!
//! A Rust implementation of GoHangout, a Logstash alternative for ETL processing.

use anyhow::{Context, Result};
use gohangout_rs::event::Event;
use gohangout_rs::input::{StdinInput, RandomInput};
use gohangout_rs::output::StdoutOutput;
use gohangout_rs::plugin::{Plugin, PluginConfig, PluginFactory, PluginType, Input, Output};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

/// Main entry point for GoHangout-rs
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 GoHangout-rs v0.1.0");
    println!("📊 A Rust implementation of GoHangout (Logstash alternative)");
    println!();
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }
    
    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: Missing config file path");
                print_usage();
                std::process::exit(1);
            }
            let config_path = &args[2];
            run_with_config(config_path).await
        }
        "demo" => run_demo().await,
        "test" => run_tests().await,
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "version" | "--version" | "-v" => {
            println!("GoHangout-rs v0.1.0");
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage();
            std::process::exit(1)
        }
    }
}

/// Print usage information
fn print_usage() {
    println!("Usage: gohangout-rs <command> [options]");
    println!();
    println!("Commands:");
    println!("  run <config.yaml>    Run with configuration file");
    println!("  demo                 Run demonstration pipeline");
    println!("  test                 Run internal tests");
    println!("  help                 Show this help message");
    println!("  version              Show version information");
    println!();
    println!("Examples:");
    println!("  gohangout-rs run config.yaml");
    println!("  gohangout-rs demo");
    println!("  gohangout-rs test");
}

/// Run GoHangout with a configuration file
async fn run_with_config(config_path: &str) -> Result<()> {
    println!("📁 Loading configuration from: {}", config_path);
    
    // Note: Configuration loading from file is not yet implemented
    // This is a placeholder for future implementation
    println!("⚠️  Configuration file loading is not yet implemented");
    println!("📊 Using demonstration mode instead");
    println!();
    
    // Fall back to demonstration mode
    run_demo().await
}

/// Run a demonstration pipeline
async fn run_demo() -> Result<()> {
    println!("🎬 Running demonstration pipeline");
    println!("📊 This demonstrates the basic ETL capabilities");
    println!();
    
    // Create a simple pipeline manually
    println!("1. Creating RandomInput plugin...");
    let mut random_config = HashMap::new();
    random_config.insert("from".to_string(), json!(1));
    random_config.insert("to".to_string(), json!(100));
    random_config.insert("max_messages".to_string(), json!(10));
    
    let random_plugin_config = PluginConfig::new("demo_random", PluginType::Input);
    let mut random_plugin_config = random_plugin_config;
    random_plugin_config.config = random_config;
    
    let mut random_input = RandomInput::from_config(&random_plugin_config)
        .context("Failed to create RandomInput")?;
    random_input.initialize()?;
    
    println!("2. Creating StdoutOutput plugin...");
    let mut stdout_config = HashMap::new();
    stdout_config.insert("format".to_string(), json!("pretty"));
    stdout_config.insert("color".to_string(), json!(true));
    
    let stdout_plugin_config = PluginConfig::new("demo_stdout", PluginType::Output);
    let mut stdout_plugin_config = stdout_plugin_config;
    stdout_plugin_config.config = stdout_config;
    
    let mut stdout_output = StdoutOutput::from_config(&stdout_plugin_config)
        .context("Failed to create StdoutOutput")?;
    stdout_output.initialize()?;
    
    println!("3. Processing 10 random events...");
    println!("────────────────────────────────");
    
    for i in 1..=10 {
        // Read from random input
        let event_opt = random_input.read()
            .context("Failed to read from RandomInput")?;
        
        if let Some(mut event) = event_opt {
            // Add some metadata
            let mut data = event.data_mut();
            if let Some(obj) = data.as_object_mut() {
                obj.insert("demo_id".to_string(), json!(i));
                obj.insert("source".to_string(), json!("demo"));
                obj.insert("timestamp".to_string(), json!(chrono::Utc::now().to_rfc3339()));
            }
            
            // Write to stdout
            stdout_output.write(event)
                .context("Failed to write to StdoutOutput")?;
            
            // Small delay for demonstration
            time::sleep(Duration::from_millis(100)).await;
        } else {
            break;
        }
    }
    
    println!("────────────────────────────────");
    println!("4. Cleaning up...");
    
    random_input.shutdown()?;
    stdout_output.shutdown()?;
    
    println!("✅ Demonstration completed successfully!");
    println!("📊 Statistics:");
    println!("   - RandomInput: {} events read", random_input.stats().events_read);
    println!("   - StdoutOutput: {} events written", stdout_output.stats().events_written);
    
    Ok(())
}

/// Run internal tests
async fn run_tests() -> Result<()> {
    println!("🧪 Running internal tests...");
    println!();
    
    // Test 1: RandomInput basic functionality
    println!("1. Testing RandomInput...");
    {
        let mut config = HashMap::new();
        config.insert("from".to_string(), json!(1));
        config.insert("to".to_string(), json!(10));
        
        let mut plugin_config = PluginConfig::new("test_random", PluginType::Input);
        plugin_config.config = config;
        
        let mut random = RandomInput::from_config(&plugin_config)?;
        random.initialize()?;
        
        for _ in 0..5 {
            let event_opt = random.read()?;
            assert!(event_opt.is_some());
            let event = event_opt.unwrap();
            let data = event.data();
            let message = data.get("message").unwrap().as_str().unwrap();
            let num: i64 = message.parse().unwrap();
            assert!(num >= 1 && num <= 10);
        }
        
        random.shutdown()?;
        println!("   ✅ RandomInput test passed");
    }
    
    // Test 2: StdoutOutput basic functionality
    println!("2. Testing StdoutOutput...");
    {
        let mut config = HashMap::new();
        config.insert("format".to_string(), json!("json"));
        
        let mut plugin_config = PluginConfig::new("test_stdout", PluginType::Output);
        plugin_config.config = config;
        
        let mut stdout = StdoutOutput::from_config(&plugin_config)?;
        stdout.initialize()?;
        
        let mut event_data = HashMap::new();
        event_data.insert("message".to_string(), json!("Test message"));
        event_data.insert("level".to_string(), json!("INFO"));
        let event = Event::new(serde_json::Value::Object(event_data.into_iter().collect()));
        
        stdout.write(event)?;
        stdout.flush()?;
        stdout.shutdown()?;
        
        println!("   ✅ StdoutOutput test passed");
    }
    
    // Test 3: Plugin factory
    println!("3. Testing PluginFactory...");
    {
        let mut factory = PluginFactory::new();
        
        factory.register_input("random", || {
            Ok(Box::new(RandomInput::default()) as Box<dyn gohangout_rs::plugin::Input>)
        });
        
        factory.register_output("stdout", || {
            Ok(Box::new(StdoutOutput::default()) as Box<dyn gohangout_rs::plugin::Output>)
        });
        
        assert!(factory.supports_plugin("random", PluginType::Input));
        assert!(factory.supports_plugin("stdout", PluginType::Output));
        
        println!("   ✅ PluginFactory test passed");
    }
    
    println!();
    println!("🎉 All tests passed successfully!");
    
    Ok(())
}

/// Example configuration file content
#[allow(dead_code)]
fn example_config() -> String {
    r#"# Example GoHangout-rs configuration
inputs:
  - type: random
    name: random_input
    config:
      from: 1
      to: 1000
      max_messages: 100

filters:
  - type: add_field
    name: add_timestamp
    config:
      field: "@timestamp"
      value: "{{timestamp}}"

outputs:
  - type: stdout
    name: console_output
    config:
      format: json
      pretty: true
"#.to_string()
}