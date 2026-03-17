# Codex Review Prompt Template

Codex に渡すプロンプトのテンプレート。
変数は実行時に置換する。

## PR スコープ用テンプレート

```
You are reviewing Pull Request #{PR_NUMBER} in the following project:
{PROJECT_DESCRIPTION}

Review the following diff and report findings organized by perspective.

Perspectives to check:
{PERSPECTIVES}

For each finding, report:
- Perspective: (architecture|security|performance|reliability|maintainability)
- Severity: (Must Fix|Should Fix|Suggestion)
- File: path/to/file
- Line: line number
- Title: short description
- Issue: what the problem is
- Fix: how to fix it

Diff:
{DIFF}
```

## プロジェクトスコープ用テンプレート

```
You are performing a full project review of the following project:
{PROJECT_DESCRIPTION}

Scan the codebase and report findings organized by perspective.

Perspectives to check:
{PERSPECTIVES}

IMPORTANT: Only report findings for the perspectives listed above. Skip any perspective not in the list.

Focus areas per perspective (include only those matching {PERSPECTIVES}):
- Architecture: layer violations, dependency inversions, module boundary violations
- Security: injection, auth bypass, hardcoded secrets, input validation
- Performance: N+1 queries, resource leaks, missing pagination
- Reliability: exception swallowing, transaction boundary issues, data integrity
- Maintainability: naming, code duplication, complexity

For each finding, report:
- Perspective: (architecture|security|performance|reliability|maintainability)
- Severity: (Must Fix|Should Fix|Suggestion)
- File: path/to/file
- Line: line number
- Title: short description
- Issue: what the problem is
- Fix: how to fix it
```

## 変数一覧

| 変数 | 説明 | スコープ |
|------|------|----------|
| `{PR_NUMBER}` | PR 番号 | PR |
| `{PROJECT_DESCRIPTION}` | プロジェクト説明（`skill_config.json` の `projectDescription`） | 共通 |
| `{PERSPECTIVES}` | 有効な観点のカンマ区切りリスト | 共通 |
| `{DIFF}` | PR の差分内容 | PR |
