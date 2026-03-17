---
name: review
description: PRのコードレビューを多観点で実行するスキル。/review または「レビュー」「コードレビュー」「セキュリティレビュー」などのリクエストで使用。アーキテクチャ・セキュリティ・パフォーマンス・信頼性・保守性の観点でレビューし、gh CLIでPRにコメントを投稿する。Codex連携によるクロスバリデーションも可能。PR番号を引数で指定可能。
---

# Review（多観点レビュー + Codex 連携）

PRの差分またはプロジェクト全体を、複数の観点（architecture / security / performance / reliability / maintainability）でレビューし、結果をPRコメントまたはファイルとして出力する。
オプションで Codex を並行起動し、クロスバリデーションを行う。

## 設定の読み込み

実行前にスキル固有設定 `skill_config.json` とグローバル設定 `.claude/skills/skills_config.json` を読み込み、以下の値を使用する。

> **設定ファイルの使い分け**: スキル固有の設定は `.claude/skills/review/skill_config.json` に、プロジェクト共通の設定は `.claude/skills/skills_config.json` に格納されている。

| 設定キー | 設定ファイル | 用途 | 必須 |
|----------|-------------|------|------|
| `docs.bestPractices` | `skills_config.json` | レビュー観点として読み込むベストプラクティスファイルのパス配列 | No |
| `excludePatterns` | `skill_config.json` | レビュー対象外とするファイルの glob パターン配列 | No |
| `additionalChecks` | `skill_config.json` | ファイルパターン別の追加チェック項目配列 | No |
| `perspectives` | `skill_config.json` | 有効な観点のリスト（デフォルト: 全5観点） | No |
| `codex.enabled` | `skill_config.json` | Codex 連携（`"auto"` / `true` / `false`） | No |
| `codex.promptTemplate` | `skill_config.json` | Codex 用プロンプトテンプレートのパス | No |
| `projectScope.outputDir` | `skill_config.json` | プロジェクトスコープ時の出力先ディレクトリ | No |
| `projectDescription` | `skill_config.json` | Codex テンプレートに渡すプロジェクト説明（デフォルト: `"Software project"`） | No |

### excludePatterns の例

```json
[
  "**/build/**",
  "**/generated/**",
  "**/dist/**",
  "*.min.js"
]
```

### additionalChecks の構造

```json
[
  {
    "pattern": "*.sql",
    "check": "SQL構文の正しさと SQLインジェクションリスクを確認する"
  },
  {
    "pattern": "V*__.sql",
    "check": "マイグレーションファイルの命名規則と既存ファイルの未変更を確認する"
  }
]
```

## 使用方法

```
/review [pr-number] [--perspective <list>] [--scope pr|project] [--codex]
```

- `pr-number`: 対象PR番号（省略時: 現在ブランチのPR）
- `--perspective <list>`: レビュー観点の絞り込み（カンマ区切り）。省略時は `skill_config.json` の `perspectives` 全て
  - 指定可能な値: `architecture`, `security`, `performance`, `reliability`, `maintainability`
- `--scope pr|project`: レビュー範囲（デフォルト: `pr`）
  - `pr`: PR差分のみレビュー → 結果をPR Reviewとして投稿
  - `project`: プロジェクト全体をスキャン → 結果を `projectScope.outputDir` に保存
- `--codex`: Codex クロスバリデーションを強制実行

### 後方互換性

引数なし `/review <pr-number>` は従来と同等の動作（全観点・PRスコープ・Codex自動検出）。
`/resolve-issue` Phase 4 からの呼び出しもそのまま動作する。

### 自然言語トリガー

「セキュリティレビューして」→ `--perspective security` として解釈。
「コードレビューして」→ 全観点として解釈。

### レビューモード

| モード | 使い方 | コンテキスト範囲 |
|--------|--------|------------------|
| **簡易** | `/review 244`（他ブランチから実行） | PR差分のみ（`gh pr diff` で取得） |
| **詳細** | 対象ブランチにチェックアウト後 `/review` | PR差分 + ローカルファイル全体 |

差分だけでは文脈が不足する場合（関数の呼び出し元、型定義の全体像、既存パターンとの整合性など）、対象ブランチにチェックアウトしてからレビューを実行する。

## 実行手順

### Phase 0: 引数パースとデフォルト解決

1. 引数から `pr-number`, `--perspective`, `--scope`, `--codex` を抽出する
2. 自然言語の場合は意図を解釈し、対応するオプションにマッピングする
   - 「セキュリティレビュー」→ `--perspective security`
   - 「アーキテクチャレビュー」→ `--perspective architecture`
   - 「パフォーマンスレビュー」→ `--perspective performance`
3. 未指定のオプションにデフォルト値を適用:
   - `perspectives`: `skill_config.json` の `perspectives` 全て
   - `scope`: `pr`
   - `codex`: `skill_config.json` の `codex.enabled`（デフォルト `"auto"`）

### Phase 1: Codex 起動（条件付き）

Codex の利用可否を判定し、条件を満たす場合はバックグラウンドで起動する。

#### 1.1 Codex 利用可否判定

```bash
which codex 2>/dev/null
```

| `codex.enabled` | Codex インストール済み | 動作 |
|-----------------|----------------------|------|
| `"auto"` | あり | 起動する |
| `"auto"` | なし | スキップ（Claude-only） |
| `true` / `--codex` | あり | 起動する |
| `true` / `--codex` | なし | エラー報告（Codex未インストール） |
| `false` | - | スキップ |

#### 1.2 Codex バックグラウンド起動

`references/codex-prompt-template.md` からプロンプトテンプレートを読み込み、変数を置換して実行する。

**PR スコープの場合:**

```bash
# PR 差分を取得してプロンプトを一時ファイルに書き出し（ARG_MAX 制限を回避）
gh pr diff <pr-number> > /tmp/codex-diff-<pr-number>.txt
# Write ツールでプロンプトファイルを作成（テンプレート変数を置換済み）
# ファイルパス: /tmp/codex-prompt-<pr-number>.txt
cat /tmp/codex-diff-<pr-number>.txt >> /tmp/codex-prompt-<pr-number>.txt
codex review "$(cat /tmp/codex-prompt-<pr-number>.txt)" 2>&1 | tee /tmp/codex-review-<pr-number>.md
# 差分が大きい場合は stdin 経由: cat /tmp/codex-prompt.txt | codex review
```

**プロジェクトスコープの場合:**

```bash
REVIEW_DIR="<projectScope.outputDir>"
mkdir -p "$REVIEW_DIR"
# Write ツールでプロンプトファイルを作成
codex review "$(cat /tmp/codex-prompt.txt)" 2>&1 | tee ${REVIEW_DIR}/codex-review.md
```

> **注意**: PR 差分が大きい場合、コマンドライン引数の長さ制限（ARG_MAX）に達する可能性がある。その場合は一時ファイルや stdin 経由でプロンプトを渡すこと。

Bash ツールの `run_in_background: true` で起動し、Claude のレビューと並行して実行する。

**Codex が失敗した場合**: ログにエラーを記録し、Claude-only で続行する。

### Phase 2: コンテキスト収集

#### PR スコープ（`--scope pr`）

```bash
# PR情報の取得
gh pr view <pr-number> --json number,url,title,state,baseRefName,headRefName

# 差分サマリ
gh pr diff <pr-number> --name-only

# 差分詳細
gh pr diff <pr-number>
```

PRが存在しない、またはクローズ済みの場合はエラー報告して終了。

差分が大きい場合はファイル単位で分割取得する。
`excludePatterns` にマッチするファイルはレビュー対象から除外する。

**詳細モード時の追加コンテキスト取得**（現在のブランチが対象PRのブランチと一致する場合）:
- 変更された関数の呼び出し元・定義元
- import している型定義やユーティリティの実装
- 既存の類似パターン（同ディレクトリ内の他ファイル）

#### プロジェクトスコープ（`--scope project`）

レビュー対象ファイルを列挙する:

Glob ツールまたは find でマルチモジュール構成に対応したパターンで列挙する:

```bash
# M5 マルチモジュール構成: 各モジュール配下の src を横断検索
# -type f を全拡張子に適用するためグルーピング
find . -type f \( -path '*/src/main/java/*.java' -o -path '*/src/main/resources/META-INF/**/*.sql' \) | head -200
```

または Glob ツールで `**/src/main/java/**/*.java` パターンを使用する。

`excludePatterns` にマッチするファイルは除外する。
ファイル数が多い場合は、`perspectives` に応じて優先度の高いファイルを選定する。

### Phase 3: 観点別レビュー

各 `perspective` に対応する `references/perspective-<name>.md` を読み込み、チェックリストに基づいてレビューする。

#### 3.1 観点チェックリストの読み込み

指定された各観点について、対応するチェックリストファイルを Read ツールで読み込む:

- `references/perspective-architecture.md` — レイヤー違反、依存逆転、モジュール境界
- `references/perspective-security.md` — インジェクション、認証バイパス、秘密情報
- `references/perspective-performance.md` — N+1クエリ、リソースリーク、ページネーション
- `references/perspective-reliability.md` — 例外処理、トランザクション境界、データ整合性
- `references/perspective-maintainability.md` — 命名、コード重複、複雑度

#### 3.2 レビュー実施

差分（または対象ファイル）の各ファイルに対し、読み込んだチェックリストを適用する。

**共通観点**（チェックリストに加えて常に確認）:
- `docs.bestPractices` に列挙されたファイルを読み込み、レビュー観点として使用する
- 対象ファイルのパス・構成に応じて該当するベストプラクティスファイルを読み込む
- `additionalChecks` にマッチするファイルは追加のチェック項目を適用する

#### 3.3 指摘の分類

各指摘に以下のメタデータを付与する:

| フィールド | 説明 |
|-----------|------|
| `perspective` | 観点名（architecture / security / performance / reliability / maintainability） |
| `severity` | 重大度（Must Fix / Should Fix / Suggestion） |
| `file` | 対象ファイルパス |
| `line` | 対象行番号 |
| `title` | 指摘タイトル |
| `issue` | 問題の説明 |
| `fix` | 修正案 |

### Phase 4: Codex 結果収集・マージ

Codex をバックグラウンドで起動していた場合、結果を収集してマージする。

#### 4.1 結果収集

Codex のバックグラウンドタスクが完了しているか確認する。
完了していれば結果ファイルを読み込む。まだ実行中の場合は完了を待つ。

#### 4.2 結果マージ（クロスバリデーション）

Claude と Codex の指摘をファイル名 + 行番号 + キーワードで突合する:

- **Both agents**: 両方が検出した指摘 → 信頼性が高い
- **Claude only**: Claude のみが検出した指摘
- **Codex only**: Codex のみが検出した指摘

マージ結果をサマリーの Cross-validation セクションに反映する。

### Phase 5: 出力

#### PR スコープ → PR Review 投稿

##### A. サマリー（Review Body）

PR Review の本文として投稿する。全体像を一目で把握するための要約。

```markdown
## PR Review: #<pr-number> <title>

### Summary

<PRの変更概要を1-2行で記載>

### Review Result

| Perspective | Must Fix | Should Fix | Suggestion |
|-------------|----------|------------|------------|
| Architecture | <件数> | <件数> | <件数> |
| Security | <件数> | <件数> | <件数> |
| Performance | <件数> | <件数> | <件数> |
| Reliability | <件数> | <件数> | <件数> |
| Maintainability | <件数> | <件数> | <件数> |

### Codex Cross-validation (Codex 使用時のみ)

- Both agents: N findings
- Claude only: N findings
- Codex only: N findings

### Good Points

<良い実装や工夫があれば記載>

---
Reviewed with [Claude Code](https://claude.com/claude-code)
```

指摘が 0 件の観点は Review Result テーブルから行ごと省略する。
Codex 未使用時は Cross-validation セクションを省略する。
観点を絞り込んでいる場合は、指定された観点の行のみ表示する。

##### B. 個別指摘（Review Comments）

各指摘を対象ファイルの該当行に紐付ける Review Comment として構成する。

各コメントの形式:

```markdown
<severity-emoji> **[<Perspective>] <指摘タイトル>**

**Issue**: <問題の説明>

**Fix**: <修正案>
```

severity-emoji は `🔴 Must Fix` / `🟡 Should Fix` / `🟢 Suggestion` のいずれか。
`[Perspective]` タグにより、どの観点からの指摘かを明示する。

##### C. 差分の行番号マッピング

Review Comment を投稿するには、diff 上の相対位置（`line`）を指定する必要がある。

1. `gh pr diff <pr-number>` の出力を解析する
2. 各指摘に対し、対象ファイルの **diff 上の行番号**（変更後ファイルの行番号）を特定する
3. 指摘対象が diff に含まれない行の場合、最も近い変更行を使うか、ファイルレベルコメントにする

##### D. PRへのレビュー投稿

`gh api` で Pull Request Review を作成する。サマリーと全指摘を一括投稿する。

```bash
# 1. レビューデータを JSON ファイルに書き出し（Write ツールを使用）
#    ファイルパス: tmp-review-payload.json
#
#    JSON 構造:
#    {
#      "event": "COMMENT",
#      "body": "<サマリー（Markdown）>",
#      "comments": [
#        {
#          "path": "<file-path>",
#          "line": <変更後ファイルの行番号>,
#          "side": "RIGHT",
#          "body": "<個別指摘（Markdown）>"
#        },
#        ...
#      ]
#    }
#
#    注意:
#    - event は "COMMENT"（情報提供）を使用する。"REQUEST_CHANGES" は使わない
#    - path はリポジトリルートからの相対パス
#    - line は変更後ファイルの行番号（diff hunk 内の行）
#    - side は "RIGHT"（変更後側）を指定する

# 2. gh api で Review を作成
gh api repos/{owner}/{repo}/pulls/<pr-number>/reviews \
  --input tmp-review-payload.json

# 3. 一時ファイルを削除
rm tmp-review-payload.json
```

指摘が 0 件の場合は `comments` を空配列にし、サマリーのみ投稿する。

#### プロジェクトスコープ → ファイル保存

結果を `projectScope.outputDir` に保存する。

```bash
# レビュー番号の決定
NEXT=$(printf "%04d" $(( $(ls -d <projectScope.outputDir>/[0-9]* 2>/dev/null | wc -l) + 1 )))
DATE=$(date +%Y-%m-%d)
REVIEW_DIR="<projectScope.outputDir>/${NEXT}-${DATE}"
mkdir -p "$REVIEW_DIR"
```

保存ファイル:
- `${REVIEW_DIR}/claude-review.md` — Claude のレビュー結果
- `${REVIEW_DIR}/codex-review.md` — Codex のレビュー結果（Codex 使用時のみ）
- `${REVIEW_DIR}/summary.md` — サマリー（Cross-validation 含む）

PRコメントは投稿しない。

### Phase 6: 結果報告

ユーザーに以下を報告する:

**PR スコープの場合:**
- 投稿した Review の URL
- 観点別の指摘件数サマリー
- Codex Cross-validation 結果（使用時）

**プロジェクトスコープの場合:**
- 保存先ディレクトリのパス
- 観点別の指摘件数サマリー
- Codex Cross-validation 結果（使用時）

## 注意事項

- レビュー対象は差分のみ（PRスコープ時）。差分外の既存コードへの指摘は行わない
- 指摘件数が 0 件の場合も「問題なし」としてコメントを投稿する
- `excludePatterns` にマッチするファイルはレビュー対象外
- `additionalChecks` にマッチするファイルは追加のチェック項目を適用する
- ベストプラクティスファイルは対象ファイルが存在する場合のみ読み込む
- Codex が利用できない環境では Claude-only で完全に動作する（graceful degradation）
- `--perspective` で観点を絞り込んだ場合、指定外の観点はスキップする
- `/review-respond` との互換性のため、severity emoji（🔴🟡🟢）のフォーマットは維持する
