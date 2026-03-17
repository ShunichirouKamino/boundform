---
name: review-respond
description: PRレビュー指摘に対応するスキル。/review-respond または「レビュー対応」「指摘対応」などのリクエストで使用。現在のブランチのPRからレビューコメントを取得し、コード修正後に対応結果をPRコメントとして返信。
---

# Review Respond

PRのレビュー指摘事項を取得し、対応後にコメントで返信する。

## 設定の読み込み

実行前に `.claude/skills/skills_config.json` を読み込み、以下の値を使用する。ファイルが存在しない場合はこのドキュメント内のデフォルト値を使用する。

| 設定キー | 用途 | デフォルト値 |
|----------|------|-------------|
| `build.fastCommand` | ビルド確認コマンド（`/pre-commit` 経由） | `./gradlew spotlessApply spotlessCheck test assemble` |

## 使用方法

```
/review-respond
```

## 実行手順

### 1. PR情報の取得

現在のブランチに紐づくPRを特定する。

```bash
# 現在のブランチ名を取得
git branch --show-current

# 現在のブランチのPR情報を取得
gh pr view --json number,url,title,state
```

PRが存在しない場合はエラーを報告して終了。

### 2. レビューコメントの取得

PRのレビューコメント（指摘事項）を取得する。

```bash
# リポジトリ情報を取得
gh repo view --json owner,name --jq '"\(.owner.login)/\(.name)"'

# PRレビューコメントを取得（行コメント）
gh api repos/<owner>/<repo>/pulls/<pr_number>/comments --jq '.[] | {id, path, line, body, user: .user.login, created_at, in_reply_to_id}'

# PR全体へのコメントも取得
gh api repos/<owner>/<repo>/issues/<pr_number>/comments --jq '.[] | {id, body, user: .user.login, created_at}'

# レビュー（Approve/Request Changes）の内容も取得
gh api repos/<owner>/<repo>/pulls/<pr_number>/reviews --jq '.[] | {id, state, body, user: .user.login}'
```

### 3. 指摘事項の分析

取得したコメントを分析し、以下を特定:

- **未対応の指摘**: 返信がないコメント、または「対応しました」等の返信がないもの
- **対応が必要なファイル**: コメントの `path` フィールドから特定
- **指摘内容**: コメントの `body` を解析

### 4. コード修正

指摘事項に基づいてコードを修正する。

1. 指摘されたファイルを Read ツールで読み込む
2. 指摘内容に従って Edit ツールで修正
3. 修正が完了したら `/pre-commit` を実行してビルド確認

### 5. 対応結果のコメント返信

コマンドインジェクションを防ぐため、コメント本文は一時ファイル経由で渡す。

```bash
# 1. コメント本文を一時ファイルに書き出し（Write ツールを使用）
#    ファイルパス: /tmp/review-comment.md

# 2. レビューコメントへの返信（行コメントの場合）
gh api repos/<owner>/<repo>/pulls/<pr_number>/comments \
  -X POST \
  -F body=@/tmp/review-comment.md \
  -F commit_id=<latest_commit_sha> \
  -F path=<file_path> \
  -F line=<line_number> \
  -F in_reply_to=<original_comment_id>

# または、PR全体へのコメント
gh pr comment <pr_number> --body-file /tmp/review-comment.md

# 3. 一時ファイルを削除
rm /tmp/review-comment.md
```

#### コメント返信の形式

```markdown
対応しました。

**修正内容:**
- <修正した内容を箇条書きで記載>

**修正コミット:** <commit_sha の短縮形>
```

### 6. 変更のコミットとプッシュ

```bash
# 変更をステージング
git add -A

# コミット（メッセージは指摘内容を要約）
git commit -m "fix: レビュー指摘対応 - <指摘内容の要約>"

# プッシュ
git push
```

### 7. 結果報告

対応した指摘事項の一覧と、コメント返信のURLをユーザーに報告する。

## 注意事項

- 複数の指摘がある場合は、1つずつ順番に対応する
- 指摘内容が不明確な場合は、ユーザーに確認を求める
- コード修正後は必ず `/pre-commit` でビルド確認を行う
- 対応不要または対応できない指摘は、その旨をコメントで返信する
