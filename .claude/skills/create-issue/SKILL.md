---
name: create-issue
description: >
  GitHub Issueを対話的に作成するスキル。リポジトリの .github/ISSUE_TEMPLATE/ からテンプレートを自動検出し、
  ユーザとの対話で要件をヒアリングしてIssueを作成する。
  /create-issue または「Issue作成」「Issue起票」「バグ報告」「機能リクエスト」などのリクエストで使用。
  引数でテンプレート名やリポジトリを指定可能（省略時はカレントリポジトリ、テンプレート一覧から選択）。
---

# Create Issue

対話的にGitHub Issueを作成する。

## 引数

```
/create-issue [テンプレート名] [--repo owner/repo]
```

- `テンプレート名`: テンプレートのファイル名部分一致（省略時: 一覧から選択）
- `--repo owner/repo`: 対象リポジトリ（省略時: カレントリポジトリ）。`owner/repo` 形式で指定する

## ワークフロー

### Phase 0: 対象リポジトリの決定

1. 引数に `--repo owner/repo` または `owner/repo` 形式の指定があればそれを使用
2. 指定がなければカレントリポジトリ（`gh repo view --json nameWithOwner -q '.nameWithOwner'`）を使用
3. 決定したリポジトリを `REPO` 変数として以降の全 Phase で使用する

**リポジトリ指定時の注意:**
- テンプレートの取得は GitHub API 経由で行う（ローカルに `.github/ISSUE_TEMPLATE/` がないため）

```bash
# リモートリポジトリからテンプレート一覧を取得
gh api repos/{REPO}/contents/.github/ISSUE_TEMPLATE --jq '.[] | select(.name != "config.yml") | .name'

# 各テンプレートファイルの内容を取得
gh api repos/{REPO}/contents/.github/ISSUE_TEMPLATE/{filename} --jq '.content' | base64 -d
```

### Phase 1: テンプレート検出と選択

1. テンプレートファイルを検出する（`config.yml` は除外）
   - **カレントリポジトリ**: `.github/ISSUE_TEMPLATE/` ディレクトリ内の `.yml` / `.yaml` ファイルを Glob で検出
   - **リモートリポジトリ**: `gh api repos/{REPO}/contents/.github/ISSUE_TEMPLATE` で検出
2. 各テンプレートファイルを読み取り、`name` と `description` を抽出する
3. テンプレート一覧をユーザに提示し選択してもらう
   - 引数でテンプレート名が指定されている場合はファイル名の部分一致で自動選択
   - テンプレートが1つしかない場合は自動選択
   - テンプレートが見つからない場合はフリーフォーマットでIssue作成に進む

### Phase 2: 要件ヒアリング

選択されたテンプレートの `body` フィールドを解析し、必要な情報を把握する。

**ヒアリングのルール:**
- フィールドを1つずつ順に聞くのではなく、「どのような要件/問題ですか？」とオープンに聞く
- ユーザの回答から、テンプレートの各フィールドにマッピングできる情報を自動抽出する
- 不足している必須情報のみ追加で質問する
- `type: markdown` のフィールドは情報表示用なので入力不要
- `type: dropdown` や `type: checkboxes` は選択肢を提示して選んでもらう（ユーザの回答から推測できる場合は確認のみ）

### Phase 3: Issue内容の組み立てと確認

1. テンプレートの各フィールドにヒアリング結果をマッピングする
2. Issueのタイトルとボディを組み立てる
   - タイトル: テンプレートの `title` プレフィックス（例: `[BUG]`）+ ユーザの要約
   - ボディ: テンプレートの `body` 構造に従い、各フィールドのラベルと回答をMarkdownで整形
3. ラベル: テンプレートの `labels` をデフォルトとし、ユーザの回答に基づき追加ラベルを提案
4. 組み立てた内容をユーザに確認してもらう

### Phase 4: Issue作成

確認後、`gh issue create` で作成する。

```bash
# カレントリポジトリの場合
gh issue create --title "<title>" --body "<body>" --label "<label1>,<label2>"

# リモートリポジトリの場合
gh issue create --repo <REPO> --title "<title>" --body "<body>" --label "<label1>,<label2>"
```

- ボディはHEREDOCで渡す
- `REPO` が指定されている場合は必ず `--repo` オプションを付与する
- assignee やmilestone はユーザが指定した場合のみ付与
- 作成後、Issue URLを返す

### Phase 5: resolve-issue の開始確認

Issue 作成後、ユーザに対して作成した Issue の対応を続けて開始するかヒアリングする。

1. 以下のメッセージをユーザに提示する:

```
Issue #<番号> を作成しました: <Issue URL>

このまま /resolve-issue で対応を開始しますか？
1. はい — このまま resolve-issue を開始する
2. いいえ — Issue 作成のみで終了する
```

2. ユーザが「はい」を選択した場合:
   - `/resolve-issue <issue-number>` を実行する（`REPO` が指定されていた場合は `--repo <REPO>` も付与）

3. ユーザが「いいえ」を選択した場合:
   - Issue URL を報告して終了する

## テンプレート body フィールドの型と処理

| type       | 処理                                           |
|------------|----------------------------------------------|
| markdown   | 入力不要。参考情報としてユーザに表示可能               |
| textarea   | ユーザの回答からマッピング。`label` をセクション見出しにする |
| input      | ユーザの回答からマッピング。短いテキスト                |
| dropdown   | 選択肢を提示。推測できれば確認のみ                    |
| checkboxes | 選択肢を提示。複数選択可能                          |

## Issue ボディの整形ルール

```markdown
### {field.label}

{ユーザの回答}
```

- 各フィールドを `###` 見出しで区切る
- `markdown` 型のフィールドはボディに含めない
- 空の回答のフィールドも省略する

## テンプレートなしの場合

`.github/ISSUE_TEMPLATE/` が存在しない、またはテンプレートファイルがない場合:
1. タイトルと本文を自由形式でヒアリング
2. `gh issue create --title "<title>" --body "<body>"` で作成
