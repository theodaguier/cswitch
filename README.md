# cswitch

CLI to switch between multiple Anthropic/Claude accounts.

## Problem

Claude Code only supports one account at a time. If you have multiple accounts (personal, work, client...), you need to manually swap credentials every time.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/theodaguier/cswitch/main/install.sh | sh
```

Or build from source:

```bash
cargo install --path .
```

## How it works

`cswitch` stores credentials for each profile and modifies Claude Code's config to point to the right account.

### Two profile types

| Type | Mechanism |
|---|---|
| **API Key** | Sets `apiKeyHelper: "cswitch emit-key"` in `~/.claude/settings.json`. Claude Code calls this command on each launch to get the active key. |
| **OAuth** | Swaps the OAuth token into `~/.claude/.credentials.json`. Removes `apiKeyHelper` from settings.json so Claude Code uses native OAuth. |

### Where data is stored

- **Secrets** (API keys, OAuth tokens) → `~/.config/cswitch/credentials.json` (mode 600, owner-only)
- **Metadata** (name, type, label, timestamps) → `~/.config/cswitch/profiles.json`

## Usage

All commands are fully interactive — just run them without arguments.

### Add a profile

```bash
$ cswitch add
  Profile name: work
  Authentication type:
  > API Key
    OAuth (login via browser)
    Import from Claude Code (existing login)
  Anthropic API key: ****
  Label (optional): Acme Corp
  ✓ Profile 'work' added.
```

### Update credentials

Run `cswitch add` with an existing profile name:

```bash
$ cswitch add
  Profile name: work
  Profile 'work' already exists. Update credentials? (y/n): y
  Authentication type: ...
  ✓ Profile 'work' updated.
```

### Switch profile

```bash
$ cswitch use
  Switch to:
  > * work (api-key) Acme Corp
      perso (oauth) Personal
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
$ cswitch remove
  Remove which profile:
  > work (api-key) Acme Corp
  Remove profile 'work'? (y/n): y
  ✓ Profile 'work' removed.
```

### Update cswitch

```bash
$ cswitch update
  ✓ cswitch updated.
```

## Switching flow in detail

### API key profiles

1. Verifies the key exists in credentials store
2. Writes `apiKeyHelper: "cswitch emit-key"` to `~/.claude/settings.json`
3. Marks the profile as active
4. On next launch, Claude Code runs `cswitch emit-key` → gets the right key

### OAuth profiles

1. Reads the stored OAuth token for the profile
2. Writes it to `~/.claude/.credentials.json`
3. Removes `apiKeyHelper` from `settings.json` so Claude Code uses OAuth
4. Marks the profile as active

### Warning: `ANTHROPIC_API_KEY`

If the `ANTHROPIC_API_KEY` environment variable is set, it overrides everything. `cswitch use` prints a warning in that case.
