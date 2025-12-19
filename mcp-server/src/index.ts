#!/usr/bin/env node
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { spawn } from "child_process";
import { homedir } from "os";
import { readFile } from "fs/promises";
import { join } from "path";

const CONFIG_PATH = join(homedir(), ".config", "ecce", "config.json");

interface EcceConfig {
  profiles: Array<{
    name: string;
    url: string;
    api_key: string;
  }>;
  active_profile: string | null;
  default_profile: string | null;
  agents: Record<string, {
    system_prompt: string;
    description?: string;
    tools?: string[];
    model?: string;
  }>;
  tasks: Record<string, {
    additional_instructions: string;
    description?: string;
  }>;
  mcp_servers: Record<string, {
    name: string;
    config: object;
  }>;
  default_agent: string | null;
  claude_executable: string | null;
}

async function runEcce(args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  return new Promise((resolve) => {
    const proc = spawn("ecce", args, {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    proc.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    proc.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    proc.on("close", (code) => {
      resolve({ stdout, stderr, code: code ?? 1 });
    });

    proc.on("error", (err) => {
      resolve({ stdout: "", stderr: err.message, code: 1 });
    });
  });
}

async function loadConfig(): Promise<EcceConfig | null> {
  try {
    const content = await readFile(CONFIG_PATH, "utf-8");
    return JSON.parse(content);
  } catch {
    return null;
  }
}

const server = new Server(
  {
    name: "ecce-mcp-server",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
      resources: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      // API Profile Tools
      {
        name: "ecce_api_list",
        description: "List all configured API profiles with their status",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_api_add",
        description: "Add a new API profile for Claude Code/Codex",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Profile name" },
            url: { type: "string", description: "API endpoint URL" },
            api_key: { type: "string", description: "API key" },
          },
          required: ["name", "url", "api_key"],
        },
      },
      {
        name: "ecce_api_switch",
        description: "Switch to a different API profile",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Profile name to switch to" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_api_delete",
        description: "Delete an API profile",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Profile name to delete" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_api_current",
        description: "Show the currently active API profile",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_api_status",
        description: "Check connectivity status of all API profiles",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_api_set_default",
        description: "Set a default API profile",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Profile name to set as default" },
          },
          required: ["name"],
        },
      },
      // Agent Tools
      {
        name: "ecce_agent_list",
        description: "List all configured agents",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_agent_add",
        description: "Add a new agent with system prompt and configuration",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Agent name" },
            system_prompt: { type: "string", description: "System prompt for the agent" },
            description: { type: "string", description: "Description of when to use this agent" },
            tools: { type: "string", description: "Comma-separated list of tools (e.g., 'Read,Grep,Glob,Bash')" },
            model: { type: "string", description: "Model to use (e.g., 'sonnet', 'opus')" },
          },
          required: ["name", "system_prompt"],
        },
      },
      {
        name: "ecce_agent_delete",
        description: "Delete an agent",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Agent name to delete" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_agent_export",
        description: "Export agents to .claude/agents/ directory",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Agent name to export (optional, exports all if not specified)" },
            user: { type: "boolean", description: "Export to user-level directory instead of project" },
          },
          required: [],
        },
      },
      {
        name: "ecce_agent_import",
        description: "Import agents from .claude/agents/ files",
        inputSchema: {
          type: "object",
          properties: {
            path: { type: "string", description: "Path to agent file to import" },
          },
          required: ["path"],
        },
      },
      {
        name: "ecce_agent_sync",
        description: "Bidirectional sync between config and agent files",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      // Task Tools
      {
        name: "ecce_task_list",
        description: "List all configured tasks",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_task_add",
        description: "Add a new task template",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Task name" },
            instructions: { type: "string", description: "Additional instructions for the task" },
            description: { type: "string", description: "Description of the task" },
          },
          required: ["name", "instructions"],
        },
      },
      {
        name: "ecce_task_delete",
        description: "Delete a task",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "Task name to delete" },
          },
          required: ["name"],
        },
      },
      // MCP Server Management Tools
      {
        name: "ecce_mcp_add",
        description: "Add an MCP server configuration to ecce",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "MCP server name" },
            json: { type: "string", description: "Server configuration as JSON (e.g., '{\"command\": \"bun\", \"args\": [\"run\", \"server.ts\"]}')" },
          },
          required: ["name", "json"],
        },
      },
      {
        name: "ecce_mcp_remove",
        description: "Remove an MCP server from ecce config",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "MCP server name to remove" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_mcp_list",
        description: "List all MCP servers in ecce config",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      {
        name: "ecce_mcp_install",
        description: "Install an MCP server to .mcp.json (local) or ~/.claude/settings.json (global)",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "MCP server name to install" },
            global: { type: "boolean", description: "Install to ~/.claude/settings.json instead of local .mcp.json" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_mcp_uninstall",
        description: "Uninstall an MCP server from .mcp.json (local) or ~/.claude/settings.json (global)",
        inputSchema: {
          type: "object",
          properties: {
            name: { type: "string", description: "MCP server name to uninstall" },
            global: { type: "boolean", description: "Uninstall from ~/.claude/settings.json instead of local .mcp.json" },
          },
          required: ["name"],
        },
      },
      {
        name: "ecce_mcp_status",
        description: "Show MCP servers status (ecce config, global settings, and local .mcp.json)",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
      // Homo (File Watching) - informational only since it's interactive
      {
        name: "ecce_homo_info",
        description: "Get information about the file watching feature and pattern syntax",
        inputSchema: {
          type: "object",
          properties: {},
          required: [],
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      // API Profile Tools
      case "ecce_api_list": {
        const result = await runEcce(["api", "list"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_api_add": {
        const { name: profileName, url, api_key } = args as { name: string; url: string; api_key: string };
        const result = await runEcce(["api", "add", profileName, "--url", url, "--key", api_key]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || "Profile added successfully" }],
        };
      }

      case "ecce_api_switch": {
        const { name: profileName } = args as { name: string };
        const result = await runEcce(["api", "switch", profileName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Switched to profile: ${profileName}` }],
        };
      }

      case "ecce_api_delete": {
        const { name: profileName } = args as { name: string };
        const result = await runEcce(["api", "delete", profileName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Deleted profile: ${profileName}` }],
        };
      }

      case "ecce_api_current": {
        const result = await runEcce(["api", "current"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_api_status": {
        const result = await runEcce(["api", "status"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_api_set_default": {
        const { name: profileName } = args as { name: string };
        const result = await runEcce(["api", "set-default", profileName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Set default profile: ${profileName}` }],
        };
      }

      // Agent Tools
      case "ecce_agent_list": {
        const result = await runEcce(["agent", "list"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_agent_add": {
        const { name: agentName, system_prompt, description, tools, model } = args as {
          name: string;
          system_prompt: string;
          description?: string;
          tools?: string;
          model?: string;
        };
        const cmdArgs = ["agent", "add", agentName, system_prompt];
        if (description) cmdArgs.push("--description", description);
        if (tools) cmdArgs.push("--tools", tools);
        if (model) cmdArgs.push("--model", model);
        const result = await runEcce(cmdArgs);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Agent '${agentName}' added successfully` }],
        };
      }

      case "ecce_agent_delete": {
        const { name: agentName } = args as { name: string };
        const result = await runEcce(["agent", "delete", agentName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Deleted agent: ${agentName}` }],
        };
      }

      case "ecce_agent_export": {
        const { name: agentName, user } = args as { name?: string; user?: boolean };
        const cmdArgs = ["agent", "export"];
        if (agentName) cmdArgs.push(agentName);
        if (user) cmdArgs.push("--user");
        const result = await runEcce(cmdArgs);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || "Agents exported successfully" }],
        };
      }

      case "ecce_agent_import": {
        const { path } = args as { path: string };
        const result = await runEcce(["agent", "import", path]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || "Agent imported successfully" }],
        };
      }

      case "ecce_agent_sync": {
        const result = await runEcce(["agent", "sync"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || "Agents synced successfully" }],
        };
      }

      // Task Tools
      case "ecce_task_list": {
        const result = await runEcce(["task", "list"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_task_add": {
        const { name: taskName, instructions, description } = args as {
          name: string;
          instructions: string;
          description?: string;
        };
        const cmdArgs = ["task", "add", taskName, instructions];
        if (description) cmdArgs.push("--description", description);
        const result = await runEcce(cmdArgs);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Task '${taskName}' added successfully` }],
        };
      }

      case "ecce_task_delete": {
        const { name: taskName } = args as { name: string };
        const result = await runEcce(["task", "delete", taskName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `Deleted task: ${taskName}` }],
        };
      }

      // MCP Server Management Tools
      case "ecce_mcp_add": {
        const { name: serverName, json } = args as { name: string; json: string };
        const result = await runEcce(["mcp", "add", serverName, json]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `MCP server '${serverName}' added` }],
        };
      }

      case "ecce_mcp_remove": {
        const { name: serverName } = args as { name: string };
        const result = await runEcce(["mcp", "remove", serverName]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `MCP server '${serverName}' removed` }],
        };
      }

      case "ecce_mcp_list": {
        const result = await runEcce(["mcp", "list"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      case "ecce_mcp_install": {
        const { name: serverName, global: isGlobal } = args as { name: string; global?: boolean };
        const cmdArgs = ["mcp", "install", serverName];
        if (isGlobal) cmdArgs.push("--global");
        const result = await runEcce(cmdArgs);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `MCP server '${serverName}' installed` }],
        };
      }

      case "ecce_mcp_uninstall": {
        const { name: serverName, global: isGlobal } = args as { name: string; global?: boolean };
        const cmdArgs = ["mcp", "uninstall", serverName];
        if (isGlobal) cmdArgs.push("--global");
        const result = await runEcce(cmdArgs);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr || `MCP server '${serverName}' uninstalled` }],
        };
      }

      case "ecce_mcp_status": {
        const result = await runEcce(["mcp", "status"]);
        return {
          content: [{ type: "text", text: result.stdout || result.stderr }],
        };
      }

      // Homo Info
      case "ecce_homo_info": {
        return {
          content: [{
            type: "text",
            text: `# ecce homo - File Watching Feature

The 'ecce homo' command watches files for special patterns and automatically triggers Claude agents to generate content.

## Pattern Types

### Inline Pattern
\`ecce <your prompt here> ecce\`

### Code Block Pattern
\`\`\`ecce
<your prompt here>
\`\`\`

## Usage
Run from terminal: \`ecce homo <file-path>\`

Options:
- \`--agent <name>\`: Use a specific agent
- \`--task <name>\`: Use a specific task template
- \`--interval <ms>\`: Set watch interval (default: 100ms)

The command will watch the file and replace patterns with AI-generated responses.

Note: This is an interactive command that runs in the terminal and cannot be executed via MCP.`,
          }],
        };
      }

      default:
        return {
          content: [{ type: "text", text: `Unknown tool: ${name}` }],
          isError: true,
        };
    }
  } catch (error) {
    return {
      content: [{ type: "text", text: `Error: ${error instanceof Error ? error.message : String(error)}` }],
      isError: true,
    };
  }
});

// List available resources
server.setRequestHandler(ListResourcesRequestSchema, async () => {
  return {
    resources: [
      {
        uri: "ecce://config",
        name: "Ecce Configuration",
        description: "Current ecce configuration including profiles, agents, tasks, and MCP servers",
        mimeType: "application/json",
      },
      {
        uri: "ecce://profiles",
        name: "API Profiles",
        description: "List of configured API profiles",
        mimeType: "application/json",
      },
      {
        uri: "ecce://agents",
        name: "Agents",
        description: "List of configured agents with their system prompts",
        mimeType: "application/json",
      },
      {
        uri: "ecce://tasks",
        name: "Tasks",
        description: "List of configured task templates",
        mimeType: "application/json",
      },
      {
        uri: "ecce://mcp-servers",
        name: "MCP Servers",
        description: "List of MCP servers saved in ecce config",
        mimeType: "application/json",
      },
    ],
  };
});

// Read resources
server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;
  const config = await loadConfig();

  if (!config) {
    return {
      contents: [{
        uri,
        mimeType: "application/json",
        text: JSON.stringify({ error: "Config not found. Run 'ecce' to initialize." }, null, 2),
      }],
    };
  }

  switch (uri) {
    case "ecce://config":
      // Return config without sensitive API keys
      const safeConfig = {
        ...config,
        profiles: config.profiles.map(p => ({
          name: p.name,
          url: p.url,
          api_key: "***hidden***",
        })),
      };
      return {
        contents: [{
          uri,
          mimeType: "application/json",
          text: JSON.stringify(safeConfig, null, 2),
        }],
      };

    case "ecce://profiles":
      return {
        contents: [{
          uri,
          mimeType: "application/json",
          text: JSON.stringify({
            profiles: config.profiles.map(p => ({
              name: p.name,
              url: p.url,
              is_active: p.name === config.active_profile,
              is_default: p.name === config.default_profile,
            })),
            active_profile: config.active_profile,
            default_profile: config.default_profile,
          }, null, 2),
        }],
      };

    case "ecce://agents":
      return {
        contents: [{
          uri,
          mimeType: "application/json",
          text: JSON.stringify({
            agents: config.agents,
            default_agent: config.default_agent,
          }, null, 2),
        }],
      };

    case "ecce://tasks":
      return {
        contents: [{
          uri,
          mimeType: "application/json",
          text: JSON.stringify({
            tasks: config.tasks,
          }, null, 2),
        }],
      };

    case "ecce://mcp-servers":
      return {
        contents: [{
          uri,
          mimeType: "application/json",
          text: JSON.stringify({
            mcp_servers: config.mcp_servers || {},
          }, null, 2),
        }],
      };

    default:
      return {
        contents: [{
          uri,
          mimeType: "text/plain",
          text: `Unknown resource: ${uri}`,
        }],
      };
  }
});

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("Ecce MCP Server running on stdio");
}

main().catch(console.error);
