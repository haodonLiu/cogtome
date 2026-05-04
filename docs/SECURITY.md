# Security Policy

## Three-Layer Defense in Depth

```
Layer 1: pre-commit hook     → blocks sensitive data from entering local commits
Layer 2: CI (GitHub Actions)  → blocks sensitive data from entering remote repo
Layer 3: scheduled audit      → detects leaks already in history
```

## Layer 1: Pre-commit Hook

The pre-commit hook uses [gitleaks](https://github.com/gitleaks/gitleaks) to scan staged files before every commit.

### Setup

```bash
# Install gitleaks
brew install gitleaks    # macOS
# or: curl -L https://git.io/gitleaks | tar -xzf - -C /usr/local/bin

# Install pre-commit (optional, manages hook lifecycle)
pip install pre-commit

# Link hook (already done in this repo)
ln -sf ../../scripts/pre-commit .git/hooks/pre-commit
```

### Custom Rules (`.gitleaks.toml`)

The project includes custom rules for:
- MiniMax API keys (`sk-cp-...`)
- Generic API keys with high entropy
- Bearer tokens in Authorization headers

### Bypass (not recommended)

```bash
git commit --no-verify
```

## Layer 2: CI Scan

GitHub Actions workflow runs on every push to `main` and on every PR.

Workflow: [`.github/workflows/secret-scan.yml`](.github/workflows/secret-scan.yml)

Cannot be bypassed without disabling the entire CI workflow.

## Layer 3: Historical Audit

Run a full repo scan:

```bash
gitleaks detect --source . --report-format json --report-path leak-report.json
```

First-time setup should run a full history scan to find existing leaks:

```bash
git filter-repo --path <leaked-file> --invert-paths   # remove file from history
```

## Environment Variables

API keys and secrets must never be hardcoded. Use environment variables:

```python
API_KEY = os.environ.get("MINIMAX_API_KEY", "")
if not API_KEY:
    raise RuntimeError("MINIMAX_API_KEY environment variable not set")
```

## If You Accidentally Commit Secrets

1. **Immediately rotate the credential** — the key is now public
2. Remove from history:
   ```bash
   # Use git filter-repo (requires repo rewrite)
   git filter-repo --path <file> --invert-paths
   ```
3. Force push:
   ```bash
   git push --force
   ```
4. Notify team and affected parties
