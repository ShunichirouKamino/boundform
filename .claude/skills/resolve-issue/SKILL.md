---
name: resolve-issue
description: 指定したGitHub Issueを読み取り、実装からPR作成・セルフレビュー・レビュー対応・Issue報告までを一貫して実行するスキル。/resolve-issue または「Issue対応」「Issue解決」などのリクエストで使用。Issue番号を引数で指定。
---

# Resolve Issue

指定した GitHub Issue の内容を読み取り、実装 → PR作成 → セルフレビュー → レビュー対応 → CI監視 → Issue報告を一貫して実行する。
現在のブランチは変更せず、git worktree を使って別ディレクトリで作業する。

## 設定の読み込み

実行前にスキル固有設定 `skill_config.json` とグローバル設定 `.claude/skills/skills_config.json` を読み込み、以下の値を使用する。ファイルが存在しない場合はこのドキュメント内のデフォルト値を使用する。

> **設定ファイルの使い分け**: スキル固有の設定は `.claude/skills/resolve-issue/skill_config.json` に、プロジェクト共通の設定は `.claude/skills/skills_config.json` に格納されている。

| 設定キー | 設定ファイル | 用途 | デフォルト値 |
|----------|-------------|------|-------------|
| `project.worktreePrefix` | `skills_config.json` | worktreeディレクトリのプレフィックス | `m5-worktree` |
| `branches.base` | `skills_config.json` | 基準ブランチ | `develop` |
| `branches.branchFormat` | `skills_config.json` | ブランチ命名形式 | `feature/issue{number}/{kebab-summary}` |
| `docs.checklists[0]` | `skills_config.json` | チェックリストパス | `doc/checklist/impl-checklist.md` |
| `docs.architecture` | `skills_config.json` | 実装ガイドパス | `["CLAUDE.md"]` |
| `docs.testPractices` | `skills_config.json` | テスト実践ガイドパス | なし |
| `docs.domainPractices` | `skills_config.json` | ドメイン知識ガイドパス | なし |
| `build.fastCommand` | `skills_config.json` | ビルド一括実行コマンド | なし |
| `ciDispatch` | `skill_config.json` | CI ワークフローの手動 dispatch を行うか | `true` |

## 使用方法

```
/resolve-issue <issue-number> [--repo owner/repo]
```

- `issue-number`: 対応する Issue 番号（必須）
- `--repo owner/repo`: 対象リポジトリ（省略時: カレントリポジトリ）。`owner/repo` 形式で指定する

### リポジトリ指定時の動作

引数に `--repo owner/repo` が指定された場合、以下のコマンドに `--repo` オプションを付与する:
- `gh issue view`, `gh issue edit`, `gh issue comment` → `--repo <REPO>`
- `gh issue develop` → `--repo <REPO>`
- `gh pr create` → `--repo <REPO>`
- `gh pr checks`, `gh pr comment` → `--repo <REPO>`
- `gh workflow run`, `gh run list` → `--repo <REPO>`

worktree の作成先は、指定リポジトリがローカルにクローン済みであることを前提とする。カレントディレクトリがそのリポジトリでない場合は、ユーザーにリポジトリのパスを確認する。

## 実行手順

### Phase 1: Issue の確認と準備

#### 1.1 Issue 内容の取得

```bash
# Issue の詳細を取得
gh issue view <issue-number> --json number,title,body,labels,assignees,state

# Issue のコメントも取得（追加要件がある場合）
gh issue view <issue-number> --comments
```

Issue がクローズ済みの場合はユーザーに報告して終了。

#### 1.2 自身を Issue にアサイン

```bash
gh issue edit <issue-number> --add-assignee @me
```

#### 1.3 worktree の作成

Issue のタイトルからブランチ名を生成し、`gh issue develop` で Issue の Development セクションに紐づいたブランチを作成した後、git worktree で別ディレクトリに作業環境を構築する。
**現在のブランチには一切影響しない。**

**命名規則**: `branches.branchFormat` に従う（デフォルト: `feature/issue{number}/{kebab-summary}`）

```bash
git fetch origin

# 1. gh issue develop でブランチを作成（Issue の Development セクションに自動リンクされる）
gh issue develop <issue-number> --name <branches.branchFormat> --base <branches.base>

# 2. リモートに作成されたブランチを取得
git fetch origin

# 3. worktree を作成（リポジトリの親ディレクトリに配置）
# WORKTREE_DIR: 以降の全 Phase でこのパスを使用する
git worktree add ../<project.worktreePrefix>-issue<番号> origin/<branches.branchFormat>
```

例: Issue #201 "ログイン画面のバリデーション修正"
- ブランチ: `feature/issue201/fix-login-validation`
- worktree: `../<project.worktreePrefix>-issue201`

**`gh issue develop` が失敗した場合のフォールバック**（ブランチ名が既に存在する等）:

```bash
# ローカルでブランチを作成し worktree を構築する（Development リンクは手動または PR の Closes キーワードで代替）
git worktree add ../<project.worktreePrefix>-issue<番号> -b <branches.branchFormat> origin/<branches.base>
```

**重要**: 以降の Phase では、`WORKTREE_DIR`（`../<project.worktreePrefix>-issue<番号>` の絶対パス）を基準にファイル操作を行うこと。

### Phase 2: 実装

#### 2.1 Issue 内容の分析

Issue の本文・ラベル・コメントから以下を特定する:

- **対応種別**: 新機能（feature）、バグ修正（bug）、改善（enhancement）、ドキュメント（documentation）等
- **対象範囲**: 変更が必要なモジュール・ファイル
- **受入基準**: Issue に記載された完了条件

#### 2.2 実装の実施

Issue の内容に基づいてコードを実装する。

- **ファイル操作**: Read / Write / Edit ツールはすべて `WORKTREE_DIR` 配下の絶対パスを使用する
- **ビルド・テスト**: `cd <WORKTREE_DIR>` で移動してからコマンドを実行する
- `docs.architecture` に列挙されたガイドの実装パターンに従う
- `docs.testPractices` が設定されている場合は参照し、変更内容に応じた適切なテスト戦略（単体テスト / 統合テスト / 両方）を選択する
- `docs.domainPractices` が設定されている場合は実装対象のドメイン知識として参照する
- 必要に応じて追加のアーキテクチャドキュメントを参照する
- 不明点がある場合はユーザーに確認を求める

#### 2.3 単体テスト作成・実行

`/unit-test` スキルを実行して、変更ファイルに対応するテストを作成・実行する。

```
Skill: unit-test
```

- 変更内容を分析し、テスト対象を自動判定する
- テスト不要と判定された場合はスキップされる

#### 2.4 統合テスト作成（API 変更がある場合）

API エンドポイントの変更がある場合、`/integration-test` スキルを実行して統合テストを作成する。

```
Skill: integration-test
```

- `/integration-test` スキルの `apiChangePatterns` に基づいて対象を判定する
- サービスが未起動の場合はテストクラス作成のみ行い、実行は後続で対応する

#### 2.5 コミット前チェック

`docs.checklists[0]` が設定されている場合はそのチェックリストに従って確認を行う。
`build.fastCommand` が設定されている場合はそれを実行してフォーマット・テスト・ビルドを一括確認する。

```bash
cd <WORKTREE_DIR>
<build.fastCommand>
```

エラーが発生した場合は修正してから次に進む。

#### 2.6 コミット

チェックが完了したら worktree ディレクトリ内で変更をコミットする。

```bash
cd <WORKTREE_DIR>
git add <変更ファイル>
git commit -m "<対応種別>: <変更内容の要約>

refs #<issue-number>"
```

#### 2.7 リモートへプッシュ

```bash
cd <WORKTREE_DIR>
git push -u origin <作成したブランチ名>
```

### Phase 3: PR 作成

**必ず Skill ツールを使って** `/create-pr` スキルを呼び出すこと。`gh pr create` を直接実行してはならない。

```
Skill: create-pr (args: <branches.base>)
```

> **重要**: `create-pr` スキルは独自のタイトルフォーマット（`[Issue#番号] 要約`）や PR 本文テンプレートを持っている。これらの規約を適用するために、必ず Skill ツール経由で実行すること。自力で `gh pr create` を実行すると、プロジェクトの PR 規約に沿わない PR が作成されてしまう。

**注意**: create-pr スキルは worktree ディレクトリのブランチ情報を使用する。worktree 内で実行すること。

#### 3.1 Issue とブランチの紐づけ確認

Phase 1.3 で `gh issue develop` を使用した場合、ブランチは既に Issue の Development セクションにリンク済み。
加えて、PR 本文に `Closes #<issue-number>` が含まれていれば、PR も自動的に Issue にリンクされる。

Phase 1.3 でフォールバック（`git worktree add -b`）を使用した場合は、PR 本文の `Closes #<issue-number>` キーワードによる自動リンクのみとなる。

### Phase 4: セルフレビュー

**必ず Skill ツールを使って** `/review` スキルを呼び出すこと。自力でレビューコメントを作成してはならない。

```
Skill: review
```

> **重要**: `review` スキルは多観点チェックリスト（architecture / security / performance / reliability / maintainability）や PR Review API 投稿のフォーマットを持っている。Skill ツール経由で実行しないとこれらが適用されない。

### Phase 5: レビュー対応

セルフレビューで指摘事項がある場合、**必ず Skill ツールを使って** `/review-respond` スキルを呼び出すこと。

```
Skill: review-respond
```

- Must Fix / Should Fix の指摘がある場合は修正を行う
- Suggestion は内容を検討し、妥当であれば対応する
- 修正後のコミット・プッシュは review-respond スキル内で行われる

**指摘が 0 件の場合はこの Phase をスキップする。**

### Phase 6: CI テスト実行・監視

#### 6.1 PR 自動トリガー CI の監視

```bash
# PR に紐づく CI チェックの状態を取得
gh pr checks <pr-number> --watch --fail-fast
```

`--watch` により CI が完了するまでポーリングする。

#### 6.2 ワークフローの手動 dispatch

`ciDispatch` が `true` の場合、`/ci-test` スキルの `skill_config.json` に定義されたワークフローを手動 dispatch する。

```
Skill: ci-test (args: <ブランチ名>)
```

`ciDispatch` が `false` の場合、または `/ci-test` スキルが未設定の場合はこのステップをスキップする。

#### 6.3 CI 失敗時の対応

CI が失敗した場合:

1. 失敗したジョブのログを確認する

```bash
# 失敗した run のログを取得
gh run view <run-id> --log-failed
```

2. 失敗原因を分析する
   - **自分の変更が原因の場合**: 修正 → コミット → プッシュし、CI を再監視する
   - **既知の不安定テスト（flaky test）や環境要因の場合**: ユーザーに状況を報告する

3. 自力で解決できない場合、ユーザーに次の選択肢を提示する

```
CI が失敗しました。
- 失敗ジョブ: <ジョブ名>
- エラー概要: <エラーの要約>

次のいずれかを選択してください:
1. CI の失敗を無視して残りのタスク（Issue 報告・worktree 削除）を続行する
2. ここで作業を中断する（PR は作成済み。手動で対応後、残りのタスクを別途実行）
```

ユーザーが「続行」を選択した場合は Phase 7 に進む。「中断」を選択した場合は Phase 9 の結果報告のみ行い、CI 失敗中である旨を含めて報告する。

#### 6.4 CI 成功時

すべての CI が成功した場合はそのまま Phase 7 に進む。

### Phase 7: Issue への報告

#### 7.1 対応内容のコメント

Issue に対応内容をコメントする。コマンドインジェクションを防ぐため、一時ファイル経由で渡す。

```bash
# 1. コメント本文を一時ファイルに書き出し（Write ツールを使用）
#    ファイルパス: /tmp/issue-comment.md
#
#    コメント形式:
#    ## 対応内容
#
#    <実装した内容を箇条書きで記載>
#
#    ## PR
#
#    #<pr-number>
#
#    ---
#    Generated with [Claude Code](https://claude.com/claude-code)

# 2. Issue にコメント
gh issue comment <issue-number> --body-file /tmp/issue-comment.md

# 3. 一時ファイルを削除
rm /tmp/issue-comment.md
```

### Phase 8: タスクリストのチェック

Issue 本文にタスクリスト（`- [ ]` 形式のチェックボックス）が含まれている場合、すべてのチェックボックスを完了状態に更新する。

```bash
# 1. Issue 本文を取得
gh issue view <issue-number> --json body -q '.body' > /tmp/issue-body.md

# 2. タスクリストの存在を確認
#    "- [ ]" が含まれていない場合はこの Phase をスキップ

# 3. すべての未完了チェックボックスを完了に置換
#    - [ ] → - [x]
#    sed や PowerShell で置換を行う

# 4. Issue 本文を更新
gh issue edit <issue-number> --body-file /tmp/issue-body.md

# 5. 一時ファイルを削除
rm /tmp/issue-body.md
```

**タスクリストがない場合はこの Phase をスキップする。**

### Phase 9: worktree 削除と結果報告

#### 9.1 worktree の削除

作業が完了したら worktree を削除する。

```bash
# worktree を削除（ブランチはリモートに残る）
git worktree remove ../<project.worktreePrefix>-issue<番号>
```

#### 9.2 結果報告

以下の情報をユーザーに報告する:

- Issue URL
- 作成したブランチ名
- PR URL
- CI 結果（成功 / 失敗 / スキップ）
- レビュー結果の概要（指摘件数と対応状況）
- Issue へのコメント URL
- タスクリストの更新有無

## 注意事項

- 各 Phase は順番に実行する。前の Phase が完了してから次に進む
- **すべてのファイル操作・ビルド・テストは worktree ディレクトリ内で行う**
- 現在のブランチ・作業ディレクトリには一切の変更を加えない
- **Phase 3〜5 では必ず Skill ツールを使って各スキルを呼び出すこと。`gh pr create` 等のコマンドを直接実行してはならない。** 各スキルはタイトルフォーマット・本文テンプレート・レビュー規約等の独自ロジックを持っており、Skill ツール経由でないと適用されない
- 実装中に Issue の要件が不明確な場合は、ユーザーに確認を求める
- ビルドエラーやテスト失敗が発生した場合は、修正してから次の Phase に進む
- Issue のラベルに `documentation` がある場合はドキュメント修正のみ、コードレビューは不要
- 大規模な変更が必要な場合は、実装前にユーザーに方針を確認する
- worktree の削除に失敗した場合（未コミットの変更がある等）はユーザーに報告する
