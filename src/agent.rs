use anyhow::{Context, Result};
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

use crate::config::{Agent, Task};

#[derive(Clone)]
struct Message {
    role: String,
    content: String,
}

pub struct ClaudeAgent {
    claude_executable: String,
    agent: Agent,
    task: Option<Task>,
    conversation_history: Vec<Message>,
}

impl ClaudeAgent {
    pub fn new(claude_executable: String, agent: Agent, task: Option<Task>) -> Self {
        Self {
            claude_executable,
            agent,
            task,
            conversation_history: Vec::new(),
        }
    }

    /// Load context files specified in the agent configuration
    fn load_context(&self) -> Result<String> {
        let mut context = String::new();

        for file_path in &self.agent.context_files {
            let content = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read context file: {}", file_path))?;

            context.push_str(&format!("\n\n--- Context from {} ---\n", file_path));
            context.push_str(&content);
        }

        Ok(context)
    }

    /// Build the prompt using the task template and question
    fn build_prompt(&self, question: &str, context: &str) -> String {
        let template = self
            .task
            .as_ref()
            .map(|t| t.template.as_str())
            .unwrap_or("Answer the following question by creating new slides that explain and elaborate on the concept.");

        // Include conversation history
        let mut prompt = String::new();

        if !self.conversation_history.is_empty() {
            prompt.push_str("## Previous Conversation:\n\n");
            for msg in &self.conversation_history {
                prompt.push_str(&format!("{}: {}\n\n", msg.role, msg.content));
            }
            prompt.push_str("---\n\n");
        }

        prompt.push_str(&format!(
            "{}\n\nContext:\n{}\n\nQuestion: {}\n\nPlease provide slide content in Markdown format.",
            template, context, question
        ));

        prompt
    }

    /// Call Claude Code executable to generate response
    pub async fn generate_response(&mut self, question: &str) -> Result<String> {
        // Load context files
        let context = self.load_context()?;

        // Build prompt with conversation history
        let user_prompt = self.build_prompt(question, &context);

        // Create a temporary file for the system prompt
        let mut system_file = NamedTempFile::new()
            .context("Failed to create temporary file for system prompt")?;
        writeln!(system_file, "{}", self.agent.system_prompt)
            .context("Failed to write system prompt to temp file")?;
        let system_path = system_file.path().to_string_lossy().to_string();

        // Call Claude Code executable
        let output = Command::new(&self.claude_executable)
            .arg("--system-prompt-file")
            .arg(&system_path)
            .arg("--")
            .arg(&user_prompt)
            .output()
            .context(format!(
                "Failed to execute Claude Code at '{}'",
                self.claude_executable
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Claude Code execution failed: {}",
                stderr
            ));
        }

        let response = String::from_utf8(output.stdout)
            .context("Failed to parse Claude Code output as UTF-8")?
            .trim()
            .to_string();

        // Save to conversation history
        self.conversation_history.push(Message {
            role: "User".to_string(),
            content: question.to_string(),
        });
        self.conversation_history.push(Message {
            role: "Assistant".to_string(),
            content: response.clone(),
        });

        Ok(response)
    }
}
