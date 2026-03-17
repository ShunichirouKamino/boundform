---
name: create-pr
description: GitHub Pull Request を作成するスキル。/create-pr または「PR作成」「プルリクエスト作成」などのリクエストで使用。ブランチ名からIssue番号を自動抽出し、タイトルとPR本文にIssueリンクを含める。PR先ブランチを引数で指定可能。
---

# Create PR

GitHub Pull Request を作成する。ブランチ名からIssue番号を自動抽出し、タイトルとPR本文に含める。

## 設定の読み込み

実行前にスキル固有設定 `skill_config.json` とグローバル設定 `.claude/skills/skills_config.json` を読み込み、以下の値を使用する。ファイルが存在しない場合はこのドキュメント内のデフォルト値を使用する。

> **設定ファイルの使い分け**: スキル固有の設定は `.claude/skills/create-pr/skill_config.json` に、プロジェクト共通の設定は `.claude/skills/skills_config.json` に格納されている。

| 設定キー | 設定ファイル | 用途 | デフォルト値 |
|----------|-------------|------|-------------|
| `branches.base` | `skills_config.json` | PR先ブランチ（引数省略時） | `develop` |
| `branches.issuePattern` | `skills_config.json` | ブランチ名からIssue番号を抽出する正規表現 | `issue(\d+)` |
| `titleFormat` | `skill_config.json` | PRタイトルの形式 | `[Issue#{issueNumber}] {summary}` |
| `assignSelf` | `skill_config.json` | 自身をアサインするか | `true` |

## 使用方法

```
/create-pr [base-branch]
```

- `base-branch`: PR先ブランチ（省略時: `branches.base` の値）

## 実行手順

### 1. 情報収集

以下を実行:

```bash
# 現在のブランチ名を取得
git branch --show-current

# ステータス確認
git status

# リモートとの同期状態確認
git fetch origin && git rev-list --left-right --count origin/<base-branch>...HEAD

# コミット履歴（base-branchから分岐後）
git log --oneline <base-branch>..HEAD

# 差分サマリ
git diff --stat <base-branch>...HEAD

# 差分詳細
git diff <base-branch>...HEAD
```

### 2. Issue番号の抽出

ブランチ名から Issue 番号を抽出する。`branches.issuePattern` の正規表現を使用する。

**パターン**: `feature/issue<番号>/xxx` または `issue<番号>` を含むブランチ名

```
feature/issue201/add-login → Issue番号: 201
issue30-feature-update → Issue番号: 30
```

抽出方法: ブランチ名から `branches.issuePattern`（デフォルト: `issue(\d+)`）で抽出。

### 3. PRタイトルとPR本文の作成

#### タイトル形式

`titleFormat` に従う。デフォルト:

```
[Issue#<番号>] <変更の要約>
```

例: `[Issue#201] ログイン機能の追加`

#### PR本文形式

````markdown
## Summary

<コミット履歴と差分から1-3行で要約>

## Related Issue

#<Issue番号>

## Changes

<差分サマリを箇条書きで記載>

## Diff Details

```diff
<git diff の主要な変更箇所を抜粋>
```

---
Generated with [Claude Code](https://claude.com/claude-code)
````

### 4. PR作成

コマンドインジェクションを防ぐため、PR本文は一時ファイル経由で渡す。

```bash
# 1. PR本文を一時ファイルに書き出し（Write ツールを使用）
#    ファイルパス: /tmp/pr-body.md

# 2. gh コマンドで PR を作成
gh pr create --base <base-branch> --assignee @me --title "<タイトル>" --body-file /tmp/pr-body.md

# 3. 一時ファイルを削除
rm /tmp/pr-body.md
```

**注意**: タイトルに特殊文字（`$`, `` ` ``, `"`, `\` など）が含まれる場合は適切にエスケープすること。

### 5. 結果報告

作成したPRのURLをユーザーに報告する。

## 注意事項

- Issue番号が抽出できない場合は、タイトルにIssue番号を含めない
- 未プッシュのコミットがある場合は、先に `git push -u origin <branch>` を実行
- PR本文の差分は主要な変更のみを抜粋（全量は長すぎる場合がある）
