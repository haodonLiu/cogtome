# Contributing to Cogtome

## Git Workflow

### Branches

- **Branch per feature**: All new work on dedicated branches
- **Branch naming**: `feat/<short-description>`, `fix/<issue>`, `chore/<description>`, `docs/<description>`
- **Branch from**: `main`

### Commits

- **Format**: `<type>(<scope>): <description>` (e.g., `feat(engine): add parallel fork support`)
- **Types**: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`
- **Rules**:
  - Subject line ≤72 chars
  - Use imperative mood ("add" not "added")
  - No dot at end of subject
  - Body explains *why*, not *what*

### Pull Requests

- **Target**: `main`
- **No direct push to main**
- **PR description must include**:
  - Summary of changes
  - Motivation/context
  - Testing performed
  - Breaking changes (if any)
- **PR title**: Same format as commit messages

### Review Requirements

- At least **1 approval** required to merge
- All CI checks must pass
- No unresolved comments

### Merge Strategies

| Scenario | Strategy |
|----------|----------|
| Feature branch to main | **Squash merge** |
| Long-running release branch | Regular merge |
| Hotfix to main | Squash merge |
| Backport PR | Regular merge |

**Rationale**: Squash keeps main history linear and clean for `cargo log`-friendly history.

## Quick Reference

```bash
# Create feature branch
git checkout main && git pull
git checkout -b feat/my-feature

# Make commits
git add -A && git commit -m "feat(scope): description"

# Push and create PR
git push -u origin feat/my-feature
# Open PR via GitHub UI

# After approval, squash merge via GitHub UI
```
