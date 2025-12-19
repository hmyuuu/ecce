# Ecce MCP Server

MCP (Model Context Protocol) server for ecce - Claude Code management tools.

## Installation

```bash
cd mcp-server
bun install
bun run build
```

## Usage

### With Claude Code

Add to your Claude Code MCP settings (`~/.claude/claude_desktop_config.json` or similar):

```json
{
  "mcpServers": {
    "ecce": {
      "command": "bun",
      "args": ["run", "/path/to/ecce/mcp-server/dist/index.js"]
    }
  }
}
```

Or if you prefer using node:

```json
{
  "mcpServers": {
    "ecce": {
      "command": "node",
      "args": ["/path/to/ecce/mcp-server/dist/index.js"]
    }
  }
}
```

## Available Tools

### API Profile Management
- `ecce_api_list` - List all configured API profiles
- `ecce_api_add` - Add a new API profile
- `ecce_api_switch` - Switch to a different profile
- `ecce_api_delete` - Delete a profile
- `ecce_api_current` - Show current active profile
- `ecce_api_status` - Check connectivity of all profiles
- `ecce_api_set_default` - Set a default profile

### Agent Management
- `ecce_agent_list` - List all configured agents
- `ecce_agent_add` - Add a new agent with system prompt
- `ecce_agent_delete` - Delete an agent
- `ecce_agent_export` - Export agents to .claude/agents/
- `ecce_agent_import` - Import agents from files
- `ecce_agent_sync` - Sync between config and agent files

### Task Management
- `ecce_task_list` - List all configured tasks
- `ecce_task_add` - Add a new task template
- `ecce_task_delete` - Delete a task

### Information
- `ecce_homo_info` - Get info about file watching feature

## Available Resources

- `ecce://config` - Full configuration (API keys hidden)
- `ecce://profiles` - API profiles list
- `ecce://agents` - Agents with system prompts
- `ecce://tasks` - Task templates

## Requirements

- `ecce` CLI must be installed and available in PATH
- Bun or Node.js runtime
