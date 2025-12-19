use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Integration tests for ecce package

#[test]
fn test_config_persistence() {
    use ecce::config::{Config, Profile};

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Create a config with profiles
    let mut config = Config::default();
    let profile = Profile {
        name: "test-profile".to_string(),
        url: "https://api.test.com".to_string(),
        key: "test-key-123".to_string(),
        service: "claude-code".to_string(),
    };

    config.profiles.push(profile);
    config.active_profile = Some("test-profile".to_string());

    // Save config
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    // Load config back
    let loaded_json = fs::read_to_string(&config_path).unwrap();
    let loaded_config: Config = serde_json::from_str(&loaded_json).unwrap();

    // Verify
    assert_eq!(loaded_config.profiles.len(), 1);
    assert_eq!(loaded_config.profiles[0].name, "test-profile");
    assert_eq!(loaded_config.active_profile, Some("test-profile".to_string()));
}

#[test]
fn test_agent_export_import_roundtrip() {
    use ecce::config::{Agent, Config};

    let temp_dir = TempDir::new().unwrap();
    let agent_file = temp_dir.path().join("test-agent.md");

    // Create an agent
    let agent = Agent {
        name: "test-agent".to_string(),
        description: Some("A test agent for integration testing".to_string()),
        system_prompt: "You are a helpful test agent.\nYou help with testing.".to_string(),
        context_files: vec![],
        tools: Some(vec!["Read".to_string(), "Write".to_string()]),
        model: Some("sonnet".to_string()),
    };

    // Export agent to file
    let content = format!(
        "---\nname: {}\ndescription: {}\ntools: {}\nmodel: {}\n---\n\n{}",
        agent.name,
        agent.description.as_ref().unwrap(),
        agent.tools.as_ref().unwrap().join(", "),
        agent.model.as_ref().unwrap(),
        agent.system_prompt
    );
    fs::write(&agent_file, content).unwrap();

    // Import agent back
    let imported_agent = Config::import_agent_from_file(&agent_file).unwrap();

    // Verify
    assert_eq!(imported_agent.name, agent.name);
    assert_eq!(imported_agent.description, agent.description);
    assert_eq!(imported_agent.system_prompt, agent.system_prompt);
    assert_eq!(imported_agent.tools, agent.tools);
    assert_eq!(imported_agent.model, agent.model);
}

#[test]
fn test_pattern_detection_workflow() {
    use ecce::pattern::{PatternDetector, PatternType};

    let detector = PatternDetector::new();

    // Test document with multiple patterns
    let text = r#"
# Document Title

Some introduction text.

ecce What is the capital of France? ecce

More content here.

```ecce
Explain quantum computing in simple terms
```

Final paragraph.
"#;

    let patterns = detector.detect_patterns(text);

    assert_eq!(patterns.len(), 2);
    assert_eq!(patterns[0].content, "What is the capital of France?");
    assert_eq!(patterns[0].pattern_type, PatternType::Inline);
    assert_eq!(patterns[1].content, "Explain quantum computing in simple terms");
    assert_eq!(patterns[1].pattern_type, PatternType::CodeBlock);
}

#[test]
fn test_pattern_deduplication() {
    use ecce::pattern::PatternDetector;

    let mut detector = PatternDetector::new();

    let text1 = "ecce What is AI? ecce";
    let text2 = "ecce What is AI? ecce"; // Same pattern

    // First detection
    let patterns1 = detector.detect_patterns(text1);
    assert_eq!(patterns1.len(), 1);

    // Mark as processed
    detector.mark_processed(&patterns1[0].content);

    // Second detection - should be empty
    let patterns2 = detector.detect_patterns(text2);
    assert_eq!(patterns2.len(), 0);
}

#[test]
fn test_config_with_multiple_entities() {
    use ecce::config::{Agent, Config, McpServer, Profile, Task};

    let mut config = Config::default();

    // Add profiles
    config.profiles.push(Profile {
        name: "profile1".to_string(),
        url: "https://api1.com".to_string(),
        key: "key1".to_string(),
        service: "claude-code".to_string(),
    });

    config.profiles.push(Profile {
        name: "profile2".to_string(),
        url: "https://api2.com".to_string(),
        key: "key2".to_string(),
        service: "claude-code".to_string(),
    });

    // Add agents
    config.agents.insert(
        "agent1".to_string(),
        Agent {
            name: "agent1".to_string(),
            description: None,
            system_prompt: "Agent 1 prompt".to_string(),
            context_files: vec![],
            tools: None,
            model: None,
        },
    );

    // Add tasks
    config.tasks.insert(
        "task1".to_string(),
        Task {
            name: "task1".to_string(),
            template: "Task 1 template".to_string(),
        },
    );

    // Add MCP servers
    config.mcp_servers.insert(
        "server1".to_string(),
        McpServer {
            name: "server1".to_string(),
            config: serde_json::json!({"command": "node"}),
        },
    );

    // Verify all entities
    assert_eq!(config.profiles.len(), 2);
    assert_eq!(config.agents.len(), 1);
    assert_eq!(config.tasks.len(), 1);
    assert_eq!(config.mcp_servers.len(), 1);

    // Test retrieval
    assert!(config.get_agent("agent1").is_some());
    assert!(config.get_task("task1").is_some());
    assert!(config.get_mcp_server("server1").is_some());
}

#[test]
fn test_profile_switching() {
    use ecce::config::{Config, Profile};

    let mut config = Config::default();

    // Add multiple profiles
    config.profiles.push(Profile {
        name: "dev".to_string(),
        url: "https://dev.api.com".to_string(),
        key: "dev-key".to_string(),
        service: "claude-code".to_string(),
    });

    config.profiles.push(Profile {
        name: "prod".to_string(),
        url: "https://prod.api.com".to_string(),
        key: "prod-key".to_string(),
        service: "claude-code".to_string(),
    });

    // Set active profile
    config.active_profile = Some("dev".to_string());
    assert_eq!(config.get_active_profile().unwrap().name, "dev");

    // Switch profile
    config.active_profile = Some("prod".to_string());
    assert_eq!(config.get_active_profile().unwrap().name, "prod");
}

#[test]
fn test_agent_with_context_files() {
    use ecce::config::Agent;

    let agent = Agent {
        name: "context-agent".to_string(),
        description: Some("Agent with context".to_string()),
        system_prompt: "You have context files".to_string(),
        context_files: vec![
            "/path/to/context1.txt".to_string(),
            "/path/to/context2.md".to_string(),
        ],
        tools: Some(vec!["Read".to_string()]),
        model: Some("opus".to_string()),
    };

    assert_eq!(agent.context_files.len(), 2);
    assert!(agent.context_files.contains(&"/path/to/context1.txt".to_string()));
}

#[test]
fn test_pattern_hash_consistency() {
    use ecce::pattern::PatternDetector;

    let mut detector1 = PatternDetector::new();
    let mut detector2 = PatternDetector::new();

    let content = "test pattern content";

    // Mark in first detector
    detector1.mark_processed(content);

    // Mark in second detector
    detector2.mark_processed(content);

    // Both should recognize it as processed
    assert!(detector1.is_processed(content));
    assert!(detector2.is_processed(content));
}

#[test]
fn test_config_default_values() {
    use ecce::config::Config;

    let config = Config::default();

    assert!(config.profiles.is_empty());
    assert!(config.active_profile.is_none());
    assert!(config.default_profile.is_none());
    assert!(config.agents.is_empty());
    assert!(config.tasks.is_empty());
    assert!(config.default_agent.is_none());
    assert!(config.claude_executable.is_none());
    assert!(config.mcp_servers.is_empty());
}

#[test]
fn test_mcp_server_json_config() {
    use ecce::config::McpServer;

    let server = McpServer {
        name: "test-mcp".to_string(),
        config: serde_json::json!({
            "command": "bun",
            "args": ["run", "server.ts"],
            "env": {
                "API_KEY": "secret"
            }
        }),
    };

    assert_eq!(server.name, "test-mcp");
    assert_eq!(server.config["command"], "bun");
    assert!(server.config["args"].is_array());
    assert_eq!(server.config["env"]["API_KEY"], "secret");
}
