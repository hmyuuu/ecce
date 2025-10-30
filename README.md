# Ecce Claude Code Ecce

**"Behold Claude Code Behold"** - A Rust CLI tool for managing agent configurations for Claude Code and Codex.

## Features

- **Profile Management**: Add, list, switch, and delete API profiles
- **Connection Testing**: Check the connectivity status of all configured profiles
- **Easy Switching**: Quickly switch between different API endpoints and keys
- **Interactive Picker**: Browse and select profiles with an intuitive interface
- **Persistent Storage**: Profiles are stored in `~/.config/ecce/config.json`
- **Claude Code Agent Integration**: Create, manage, and sync agents with Claude Code's native format
- **File Watching with Agents**: Watch files for special patterns and automatically trigger Claude Code agents to generate content

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
  - [API Profile Management](#api-profile-management)
  - [Agent Management](#agent-management)
  - [Task Management](#task-management)
  - [File Watching with Agents](#file-watching-with-agents-homo-command)
- [Configuration](#configuration)
- [How It Works](#how-it-works)

## Quick Start

```bash
# Build and install
cargo build --release
cargo install --path .

# Create an agent compatible with Claude Code
ecce agent add code-reviewer \
  --prompt "You are a senior code reviewer" \
  --description "Use proactively after writing code" \
  --tools "Read, Grep, Glob, Bash" \
  --model "sonnet"

# Export to Claude Code format
ecce agent export code-reviewer

# Now Claude Code can use this agent automatically!
# The agent is available at .claude/agents/code-reviewer.md
```

## Installation

Build the project:

```bash
cargo build --release
```

The binary will be available at `./target/release/ecce`

Optionally, install it globally:

```bash
cargo install --path .
```

Or copy the binary to a location in your PATH:

```bash
cp ./target/release/ecce /usr/local/bin/
```

## Usage

### API Profile Management

All commands for profile management are under the `api` subcommand.

#### Add a new profile

```bash
ecce api add <profile-name> --url <api-url> --key <api-key> [--service <service-type>]
```

Example:
```bash
ecce api add production --url https://api.anthropic.com --key sk-ant-xxx --service claude-code
ecce api add development --url http://localhost:8000 --key dev-key-123 --service claude-code
```

#### List all profiles

```bash
ecce api list
```

Shows all profiles with indicators for:
- `active` - Currently active profile
- `default` - Default profile (used when switching without arguments)

#### Switch to a profile

```bash
ecce api switch [profile-name]
```

If no profile name is provided:
- Uses the default profile if set
- Otherwise, shows an interactive picker

This will update the `.mise.toml` file in the current directory with the environment variables from the selected profile.

#### Delete a profile

```bash
ecce api delete <profile-name>
```

#### Show current active profile

```bash
ecce api current
```

#### Check connection status

```bash
ecce api status
```

This command checks the connectivity of all configured profiles and displays:
- Connection status (✓ Connected / ✗ Failed)
- Response time in milliseconds
- Error details if connection fails

#### Set default profile

```bash
ecce api set-default <profile-name>
```

Sets a profile as the default, which will be used when running `ecce api switch` without arguments.

### Clear default profile

```bash
ecce api clear-default
```

Removes the default profile setting.

### Agent Management

The `ecce agent` command manages agents compatible with Claude Code's native format. Agents can be stored in your JSON config and exported to `.claude/agents/*.md` files that Claude Code reads directly.

**Key Features:**
- Create agents with Claude Code-compatible fields (description, tools, model)
- Export agents to `.claude/agents/` for Claude Code integration
- Import existing Claude Code agents into your config
- Sync agents bidirectionally between config and files
- Support for both project-level and user-level agents

#### Add a new agent

```bash
ecce agent add <agent-name> \
  --prompt "System prompt for the agent" \
  --description "When to use this agent" \
  --tools "Read, Grep, Glob, Bash" \
  --model "sonnet"
```

Example with inline prompt:
```bash
ecce agent add code-reviewer \
  --prompt "You are a senior code reviewer ensuring high standards of code quality and security." \
  --description "Expert code review specialist. Use proactively after writing code." \
  --tools "Read, Grep, Glob, Bash" \
  --model "sonnet"
```

Example with prompt from file:
```bash
# Create a prompt file
cat > code-reviewer-prompt.txt << 'EOF'
You are a senior code reviewer ensuring high standards of code quality and security.

When reviewing code:
1. Check for security vulnerabilities
2. Ensure proper error handling
3. Verify code readability and maintainability
4. Look for performance issues
EOF

# Add agent using the prompt file
ecce agent add code-reviewer \
  --prompt-file code-reviewer-prompt.txt \
  --description "Expert code review specialist. Use proactively after writing code." \
  --tools "Read, Grep, Glob, Bash" \
  --model "sonnet"
```

#### List all agents

```bash
ecce agent list
# or
ecce agent ls
```

Shows all configured agents with their descriptions, tools, and models.

#### Delete an agent

```bash
ecce agent delete <agent-name>
```

#### Export agents to Claude Code format

```bash
# Export a specific agent to .claude/agents/
ecce agent export <agent-name>

# Export all agents
ecce agent export

# Export to user-level directory (~/.claude/agents/)
ecce agent export --user
```

This creates markdown files with YAML frontmatter that Claude Code can read directly.

#### Import agents from Claude Code format

```bash
# Import from .claude/agents/
ecce agent import

# Import from user-level directory (~/.claude/agents/)
ecce agent import --user
```

#### Sync agents between config and files

```bash
# Sync from files to config (import)
ecce agent sync --direction import

# Sync from config to files (export)
ecce agent sync --direction export

# Sync with user-level directory
ecce agent sync --user --direction export
```

#### Claude Code Agent Format

Agents are stored as markdown files with YAML frontmatter:

```markdown
---
name: code-reviewer
description: Expert code review specialist. Use proactively after writing code.
tools: Read, Grep, Glob, Bash
model: sonnet
---

You are a senior code reviewer ensuring high standards of code quality and security.

When invoked:
1. Run git diff to see recent changes
2. Focus on modified files
3. Begin review immediately

Review checklist:
- Code is simple and readable
- Functions and variables are well-named
- No duplicated code
- Proper error handling
- No exposed secrets or API keys
```

#### Agent Management Workflows

**Creating and Using Agents with Claude Code:**

```bash
# 1. Create an agent
ecce agent add test-runner \
  --prompt "You are a test automation specialist" \
  --description "Use proactively to run tests and fix failures" \
  --tools "Read, Bash, Edit" \
  --model "sonnet"

# 2. Export to Claude Code format
ecce agent export test-runner

# 3. Now Claude Code can use this agent automatically!
# The agent will be available in .claude/agents/test-runner.md
```

**Importing Existing Claude Code Agents:**

```bash
# 1. Create or download agent files to .claude/agents/
# 2. Import them into ecce config
ecce agent import

# 3. List to verify
ecce agent ls
```

**Syncing Agents Across Projects:**

```bash
# Export all agents to user-level directory
ecce agent sync --user --direction export

# In another project, import user-level agents
ecce agent sync --user --direction import
```

#### Agent Command Reference

| Command | Description |
|---------|-------------|
| `ecce agent add <name> --prompt "..." [options]` | Create a new agent |
| `ecce agent list` or `ecce agent ls` | List all agents |
| `ecce agent delete <name>` | Delete an agent |
| `ecce agent export [name]` | Export agent(s) to `.claude/agents/` |
| `ecce agent export --user` | Export to `~/.claude/agents/` |
| `ecce agent import` | Import from `.claude/agents/` |
| `ecce agent import --user` | Import from `~/.claude/agents/` |
| `ecce agent sync --direction import` | Sync from files to config |
| `ecce agent sync --direction export` | Sync from config to files |

**Available Options for `add` command:**
- `--prompt` (required*): System prompt for the agent
- `--prompt-file` or `-f` (required*): File containing the system prompt
  - *Either `--prompt` or `--prompt-file` must be provided (mutually exclusive)
- `--description`: When to use this agent (helps Claude Code decide when to invoke)
- `--tools`: Comma-separated list of tools (e.g., "Read, Grep, Glob, Bash")
- `--model`: Model to use (sonnet, opus, haiku, or inherit)
- `--context`: Comma-separated list of context files

### Task Management

Tasks are additional prompts that can be combined with agent prompts to provide specific instructions for different use cases.

#### Add a new task

```bash
ecce task add <task-name> --prompt "Additional instructions..."
```

Example with inline prompt:
```bash
ecce task add explain-concept \
  --prompt "Explain the following concept in detail with examples, use cases, and best practices."
```

Example with prompt from file:
```bash
# Create a task prompt file
cat > task-prompt.txt << 'EOF'
Create a comprehensive tutorial with:
1. Clear explanations
2. Code examples
3. Common pitfalls
4. Best practices
EOF

# Add task using the prompt file
ecce task add tutorial -f task-prompt.txt
```

#### List all tasks

```bash
ecce task list
# or
ecce task ls
```

#### Delete a task

```bash
ecce task delete <task-name>
```

**How Tasks Work:**
Tasks provide additional context to agents. When you select a task with `ecce homo`, the task prompt is combined with the agent's system prompt to give more specific instructions.

### File Watching with Agents (Homo Command)

The `ecce homo` command watches a file for special patterns and automatically triggers Claude Code agents to process them and append responses.

#### Patterns Syntax

You can use two pattern types in your files:

**Inline Pattern:**
```markdown
ecce what is Rust programming language? ecce
```

**Code Block Pattern:**
````markdown
```ecce
Explain async/await in Rust with examples
```
````

#### Basic Usage

```bash
# Watch a file with the default agent
ecce homo slides.md

# Watch a folder (automatically finds slides.md)
ecce homo ./my-presentation

# Interactive mode - prompts for agent and task selection
ecce homo slides.md

# Watch with a specific agent
ecce homo slides.md --agent slide-generator

# Watch with a specific agent and task template
ecce homo slides.md --agent slide-generator --task explain-concept

# Custom watch interval (in milliseconds)
ecce homo slides.md --watch-interval 1000
```

**Interactive Selection:**
When you run `ecce homo` without specifying an agent or task, you'll see:
```
🤖 Available agents:
  1. slide-generator - You are an expert presentation creator...
  2. code-explainer - You are a technical educator...

Select agent (1-2): 1

📋 Available tasks:
  0. (No task - use default)
  1. explain-concept - Explain the following concept as slides...
  2. code-example - Provide code examples with explanations...

Select task (0-2): 0
```

#### How It Works

1. **Start watching:** Run `ecce homo <file-or-folder>`
   - If you provide a folder, it automatically looks for `slides.md` in that folder
   - If no agent is specified, you'll be prompted to select one interactively
   - If no task is specified and tasks are configured, you'll be prompted to select one (or choose "No task")
2. **Add patterns:** Edit the file and add patterns like `ecce your question? ecce`
3. **Auto-process:** The agent detects the pattern, generates a response, and appends it to the file
4. **Continue:** Keep adding patterns as needed; each is processed automatically
5. **Stop:** Press `Ctrl+C` to stop watching

#### Configuration

Agents and tasks are configured in `~/.config/ecce/config.json`. Here's an example:

```json
{
  "agents": {
    "slide-generator": {
      "name": "slide-generator",
      "system_prompt": "You are an expert presentation creator...",
      "context_files": []
    }
  },
  "tasks": {
    "explain-concept": {
      "name": "explain-concept",
      "template": "Explain the following concept as slides..."
    }
  },
  "default_agent": "slide-generator"
}
```

#### Example Workflow

```bash
# 1. Create a presentation folder with slides
mkdir my-presentation
echo "# My Presentation\n\necce what is Rust? ecce" > my-presentation/slides.md

# 2. Start watching the folder (automatically finds slides.md)
ecce homo my-presentation --agent slide-generator

# 3. The agent processes the pattern and appends slides to the file
# 4. Add more patterns while it's running
# 5. Press Ctrl+C when done
```

Or with a direct file:

```bash
# 1. Create a slides file
echo "# My Presentation\n\necce what is Rust? ecce" > slides.md

# 2. Start watching the file
ecce homo slides.md --agent slide-generator
```

#### Use Cases

- **Interactive Slide Generation:** Build presentation slides incrementally
- **Documentation:** Generate explanations and examples on-the-fly
- **Code Examples:** Ask for code snippets and have them added automatically
- **Learning:** Create educational content by asking questions
- **Research:** Compile information from multiple queries

## Configuration

### Prerequisites

This tool requires [mise](https://mise.jdx.dev/) to manage environment variables.

**Note**: `ecce` cannot automatically install mise for you. You must install and configure it manually.

#### Install mise

Choose one of the following methods:

```bash
# Using the installer script (recommended)
curl https://mise.run | sh

# Or using Homebrew (macOS/Linux)
brew install mise

# Or using cargo
cargo install mise
```

#### Activate mise in your shell

After installation, activate mise by adding one of these lines to your shell configuration file (`.bashrc`, `.zshrc`, `.config/fish/config.fish`, etc.):

```bash
eval "$(mise activate bash)"  # for bash
eval "$(mise activate zsh)"   # for zsh
mise activate fish | source   # for fish
```

Restart your shell or run `source ~/.bashrc` (or equivalent) to apply the changes.

#### Verify mise installation

Before using `ecce`, verify mise is working:

```bash
mise --version
```

If this command fails, mise is not installed or not in your PATH. Please complete the mise installation before using `ecce`.

### Profile Storage

Profiles are stored in JSON format at:
```
~/.config/ecce/config.json
```

When you switch profiles, the tool updates `.mise.toml` in the current directory with:
- `ANTHROPIC_BASE_URL`
- `ANTHROPIC_API_KEY`

## Example Workflow: Creating a Slidev Presentation

This example shows how to use `ecce` to interactively build a Slidev presentation with AI assistance.

### Step 1: Set up your agent

```bash
# Create an agent for slide generation
ecce agent add slide-generator \
  --prompt "You are an expert at creating clear, concise Slidev presentations" \
  --description "Creates educational slide content" \
  --tools "Read, Write" \
  --model "sonnet"
```

### Step 2: Create a task for inline editing

```bash
# Create a task that tells the agent to replace patterns inline
cat > inline-edit-task.txt << 'EOF'
You are editing a Slidev presentation file (slides.md). When you encounter a question or prompt, you should REPLACE the ecce pattern with properly formatted slide content.

IMPORTANT INSTRUCTIONS:
1. REPLACE the entire ecce pattern (including the markers) with your response
2. DO NOT append content - edit inline where the pattern appears
3. Format your response as proper Slidev slides using markdown
4. Use slide separators (---) between slides
5. Keep slides concise and visually appealing

PATTERN FORMATS TO REPLACE:
- Inline: `ecce <question> ecce` → Replace with slide content
- Block: ```ecce\n<question>\n``` → Replace with slide content

EXAMPLE:
BEFORE: ecce what is React? ecce
AFTER:
---
# What is React?
React is a JavaScript library for building user interfaces
- Component-based architecture
- Virtual DOM for performance
---
EOF

ecce task add inline-edit -f inline-edit-task.txt
```

### Step 3: Create your presentation file

```bash
mkdir my-presentation
cat > my-presentation/slides.md << 'EOF'
---
theme: default
---

# My Tech Talk

ecce what is TypeScript? ecce

---

# Next Topic

```ecce
explain the benefits of using TypeScript
```

---
EOF
```

### Step 4: Start watching and let AI fill in the content

```bash
# Start watching the folder (automatically finds slides.md)
ecce homo my-presentation --agent slide-generator --task inline-edit

# Or use interactive mode
ecce homo my-presentation
# Select: slide-generator
# Select: inline-edit
```

### Step 5: The AI processes patterns and replaces them inline

The file is automatically updated:

```markdown
---
theme: default
---

# My Tech Talk

---

# What is TypeScript?

TypeScript is a strongly typed programming language that builds on JavaScript

- Adds static type definitions
- Catches errors at compile time
- Better IDE support and autocomplete
- Scales well for large applications

```typescript
// Type-safe code example
interface User {
  name: string;
  age: number;
}

const user: User = {
  name: "Alice",
  age: 30
};
```

---

# Next Topic

---

# Benefits of Using TypeScript

## Type Safety
- Catch errors before runtime
- Refactor with confidence

## Developer Experience
- Intelligent code completion
- Better documentation
- Easier maintenance

## Scalability
- Perfect for large codebases
- Team collaboration
- Self-documenting code

---
```

### Step 6: Continue adding more patterns

While `ecce homo` is running, you can keep editing the file and adding more patterns. Each pattern will be detected and processed automatically!

```bash
# Add more questions to your slides.md
echo "\necce how to set up a TypeScript project? ecce" >> my-presentation/slides.md

# The agent automatically processes it and replaces it with slide content
```

### Complete API Profile Workflow

```bash
# Add profiles
ecce api add prod --url https://api.anthropic.com --key sk-ant-prod-xxx
ecce api add staging --url https://staging.api.anthropic.com --key sk-ant-staging-xxx
ecce api add local --url http://localhost:8000 --key local-dev-key

# List all profiles
ecce api list

# Check connection status
ecce api status

# Switch to production (updates .mise.toml)
ecce api switch prod

# Check current profile
ecce api current

# Switch to local development
ecce api switch local

# Set a default profile (used when running `ecce api switch` without arguments)
ecce api set-default prod

# Switch to default profile (no profile name needed)
ecce api switch

# Clear the default profile
ecce api clear-default
```

## How It Works

When you switch profiles, `ecce` writes the selected profile's URL and API key to `.mise.toml` in the current directory. The mise tool automatically loads these environment variables when you're in that directory, making them available to Claude Code and other tools that use the Anthropic API.

## Development

Built with:
- **clap**: Command-line argument parsing
- **serde/serde_json**: Configuration serialization
- **reqwest**: HTTP client for connectivity checks
- **tokio**: Async runtime
- **colored**: Terminal output formatting
- **anyhow**: Error handling

## License

MIT
