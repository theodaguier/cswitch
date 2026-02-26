# cswitch

CLI to switch between multiple Anthropic/Claude accounts.

## Problem

Claude Code only supports one account at a time. If you have multiple accounts (personal, work, client...), you need to manually swap credentials every time.

## How it works

`cswitch` stores credentials for each profile in the native OS Keychain and modifies Claude Code's config to point to the right account.

### Two profile types

| Type | Mechanism |
|---|---|
| **API Key** | Sets `apiKeyHelper: "cswitch emit-key"` in `~/.claude/settings.json`. Claude Code calls this command on each launch to get the active key. |
| **OAuth** | Swaps the OAuth token into the `Claude Code-credentials` Keychain entry. Removes `apiKeyHelper` from settings.json so Claude Code uses native OAuth. |

### Where data is stored

- **Secrets** (API keys, OAuth tokens) → Native OS Keychain (macOS Keychain / Linux secret-service / Windows Credential Manager) via the `keyring` crate
- **Metadata** (name, type, label, timestamps) → `~/.config/cswitch/profiles.json` — no secrets here

## Getting started

### Build & install

```bash
cd ~/Developer/cswitch
cargo install --path .
```

With OAuth support:

```bash
cargo install --path . --features oauth
```

### Run without installing

```bash
cd ~/Developer/cswitch
cargo run -- <command>
```

For example:

```bash
cargo run -- init
cargo run -- add work --api-key --label "Acme Corp"
cargo run -- list
```

## Usage

### Add an API key profile

```bash
$ cswitch add work --api-key --label "Acme Corp"
  Anthropic API key: ****
  ✓ Profile 'work' added.
```

### Add an OAuth profile

```bash
$ cswitch add perso --oauth
  Opening browser for authentication...
  ✓ Profile 'perso' added.
```

### Import existing Claude Code credentials

```bash
$ cswitch import current-account --label "My current account"
  ✓ Imported Claude Code credentials as profile 'current-account'.
```

### Switch profile

```bash
$ cswitch use work
  ✓ Switched to 'work' (api-key).
```

### List profiles

```bash
$ cswitch list
  * work       api-key    Acme Corp
    perso      oauth      Personal
```

### Show active profile

```bash
$ cswitch current
  Active: work (api-key, sk-ant-...a8f3)
```

### Remove a profile

```bash
$ cswitch remove work
  Remove profile 'work'? (y/n): y
  ✓ Profile 'work' removed.
```

## Project structure

```
src/
├── main.rs              # Entry point + clap dispatch
├── cli.rs               # CLI definitions (clap derive)
├── commands/
│   ├── add.rs           # Add a profile
│   ├── use_profile.rs   # Switch profile
│   ├── list.rs          # List profiles
│   ├── remove.rs        # Remove a profile
│   ├── current.rs       # Show active profile
│   ├── emit_key.rs      # [hidden] Print key for apiKeyHelper
│   ├── import.rs        # Import existing credentials
│   └── init.rs          # Initial setup
├── profile.rs           # Profile model + ProfileStore (JSON)
├── keychain.rs          # Keychain abstraction (keyring crate)
├── claude_config.rs     # Read/write ~/.claude/settings.json
├── oauth.rs             # OAuth 2.0 + PKCE flow (feature flag)
└── error.rs             # Error types (thiserror)
```

## Switching flow in detail

### `cswitch use <profile>` with an API key

1. Verifies the key exists in the Keychain
2. Writes `apiKeyHelper: "cswitch emit-key"` to `~/.claude/settings.json`
3. Marks the profile as active in `profiles.json`
4. On next launch, Claude Code runs `cswitch emit-key` → gets the right key

### `cswitch use <profile>` with OAuth

1. Reads the OAuth token for the profile from the Keychain (`cswitch/<profile>-oauth`)
2. Writes it to the `Claude Code-credentials` Keychain entry
3. Removes `apiKeyHelper` from `settings.json` so Claude Code uses OAuth
4. Marks the profile as active

### Warning: `ANTHROPIC_API_KEY`

If the `ANTHROPIC_API_KEY` environment variable is set, it overrides everything. `cswitch use` prints a warning in that case.
