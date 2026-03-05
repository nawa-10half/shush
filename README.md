# kagienv

> Lock your secrets with kagienv (鍵 + env)

A local-first secret manager for developers. Built for the VibeCoding era where AI agents can accidentally push your API keys.

## The Problem

- `.env` files with API keys get accidentally committed
- AI coding agents (`git add .` → `git push`) don't know what's sensitive
- Cloud-based secret managers require trusting a third party with your keys
- `~/.aws/credentials` sits in plaintext on disk, readable by any process

## The Solution

**kagienv** keeps your real secrets in an encrypted local vault (`~/.kagienv/`), outside your project directory. Secrets are injected as environment variables only when you need them.

```bash
# Add secrets to your local vault
kagienv add OPENAI_API_KEY sk-abc123...
kagienv add AWS_ACCESS_KEY_ID AKIA...
kagienv add AWS_SECRET_ACCESS_KEY wJalr...

# Run any command with secrets injected as env vars
kagienv run npm start
kagienv run aws s3 ls

# Scan for hardcoded secrets in your codebase
kagienv scan

# Install git pre-push hook to block pushes containing secrets
kagienv install-hooks
```

## Quick Start

### Install

```bash
cargo install --path .
```

### Usage

```bash
# Store a secret
kagienv add <NAME> <VALUE>

# List stored secrets (values are never shown)
kagienv list

# Delete a secret
kagienv delete <NAME>

# Run a command with all secrets injected as environment variables
kagienv run <command...>

# Scan current directory for hardcoded secret values
kagienv scan

# Install git pre-push hook + Claude Code hooks
kagienv install-hooks
```

### Replace ~/.aws/credentials

AWS CLI prioritizes environment variables over `~/.aws/credentials`:

```bash
kagienv add AWS_ACCESS_KEY_ID "your-access-key"
kagienv add AWS_SECRET_ACCESS_KEY "your-secret-key"

# Now use kagienv instead of credentials file
kagienv run aws s3 ls
kagienv run aws sts get-caller-identity
```

**Note:** Environment variable names must be **UPPERCASE** (e.g. `AWS_ACCESS_KEY_ID`, not `aws_access_key_id`).

## How It Works

- **Encryption:** Each secret value is encrypted with [age](https://github.com/FiloSottile/age) (x25519) before storage
- **Storage:** Encrypted values stored in SQLite (`~/.kagienv/vault.db`)
- **Key protection:**
  - **macOS:** Private key stored in macOS Keychain; `identity.txt` contains only the public key
  - **Linux/Windows:** Private key encrypted with a master password (age scrypt)
  - Legacy plaintext keys are automatically migrated on first use
- **Permissions:** Vault directory (700) and key file (600) are restricted to owner only
- **Scan:** Compares actual vault values against your codebase — not pattern matching

```
~/.kagienv/              (700)
├── keys/
│   └── identity.txt   (600) public key only (macOS) or encrypted private key (Linux/Windows)
└── vault.db           SQLite with encrypted secret values
```

Set `KAGIENV_USE_PASSWORD=1` to force master-password mode on macOS (useful for testing).

## Scan & Git Hooks

`kagienv scan` detects hardcoded secrets by matching **actual vault values** against files in your codebase. This catches secrets that pattern-based scanners miss — like when an AI agent writes your API key directly into source code.

```bash
kagienv install-hooks
```

This installs:
- **Git pre-push hook** — runs `kagienv scan` before every push, blocks if secrets are found
- **Claude Code hooks** — runs `kagienv scan` before tool execution

## Status

MVP (v0.1) — core functionality is implemented and usable.

### Implemented
- Encrypted vault with age + SQLite
- `add`, `list`, `delete`, `run`, `scan`, `install-hooks` commands
- Git pre-push hook and Claude Code hook integration

### Planned
- Team sharing via public-key encryption (`kagienv share / receive`)
- `.env` file import (`kagienv import .env`)
- SQLCipher for database-level encryption

## License

MIT
