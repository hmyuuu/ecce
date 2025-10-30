# Ecce Claude Code Ecce

**"Behold Claude Code Behold"** - A Rust CLI tool for managing agent configurations for Claude Code and Codex.

## Features

- **Profile Management**: Add, list, switch, and delete API profiles
- **Connection Testing**: Check the connectivity status of all configured profiles
- **Easy Switching**: Quickly switch between different API endpoints and keys
- **Interactive Picker**: Browse and select profiles with an intuitive interface
- **Persistent Storage**: Profiles are stored in `~/.config/ecce/config.json`

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

All commands are under the `api` subcommand for profile management.

### Add a new profile

```bash
ecce api add <profile-name> --url <api-url> --key <api-key> [--service <service-type>]
```

Example:
```bash
ecce api add production --url https://api.anthropic.com --key sk-ant-xxx --service claude-code
ecce api add development --url http://localhost:8000 --key dev-key-123 --service claude-code
```

### List all profiles

```bash
ecce api list
```

Shows all profiles with indicators for:
- `active` - Currently active profile
- `default` - Default profile (used when switching without arguments)

### Switch to a profile

```bash
ecce api switch [profile-name]
```

If no profile name is provided:
- Uses the default profile if set
- Otherwise, shows an interactive picker

This will update the `.mise.toml` file in the current directory with the environment variables from the selected profile.

### Delete a profile

```bash
ecce api delete <profile-name>
```

### Show current active profile

```bash
ecce api current
```

### Check connection status

```bash
ecce api status
```

This command checks the connectivity of all configured profiles and displays:
- Connection status (✓ Connected / ✗ Failed)
- Response time in milliseconds
- Error details if connection fails

### Set default profile

```bash
ecce api set-default <profile-name>
```

Sets a profile as the default, which will be used when running `ecce api switch` without arguments.

### Clear default profile

```bash
ecce api clear-default
```

Removes the default profile setting.

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

## Example Workflow

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
