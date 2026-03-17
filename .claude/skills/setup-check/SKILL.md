---
name: setup-check
description: スキルの設定状況を診断し、不足している設定・ファイルを特定してセットアップを支援するスキル。/setup-check または「設定確認」「スキル診断」「セットアップ」などのリクエストで使用。初回導入時や設定不備でスキルが動かない場合に実行する。
---

# Setup Check

全スキルの設定ファイル・依存ファイルを検査し、不足を報告した上で、ユーザーヒアリングとリポジトリ調査に基づいて設定を自動生成する。

## 使用方法

```
/setup-check [skill-name]
```

- `skill-name`: 特定スキルのみ診断（省略時: 全スキル一括診断）

## 実行手順

### Phase 1: 設定ファイルの存在チェック

#### 1.1 グローバル設定の確認

```bash
# skills_config.json の存在確認
cat .claude/skills/skills_config.json
```

存在しない場合:

```
❌ .claude/skills/skills_config.json が見つかりません。
   → `npx @dentsusoken/m5-claude-config init` で初期セットアップを実行してください。
```

#### 1.2 各スキルの skill_config.json 確認

`.claude/skills/*/SKILL.md` を Glob で検出し、各スキルについて:

1. SKILL.md を読み込み、設定テーブルから必須設定キーを抽出する
2. `skill_config.json` の存在を確認する
3. 存在する場合は中身を読み込み、必須キーの有無をチェックする

### Phase 2: 依存ファイル・リソースの検査

各スキルの SKILL.md に記載された外部参照ファイルの存在を検査する。

#### 2.1 検査対象

| スキル | 検査対象 | 設定キー |
|--------|----------|----------|
| review | ベストプラクティスファイル | `docs.bestPractices` |
| pre-commit | チェックリスト | `docs.checklists` |
| pre-commit | 技術スタックドキュメント | `techStackDoc` |
| resolve-issue | 実装ガイド | `docs.architecture` |
| resolve-issue | テスト実践ガイド | `docs.testPractices` |
| resolve-issue | ドメイン知識ガイド | `docs.domainPractices` |
| ci-test | GitHub Actions ワークフローファイル | `workflows[].file` |
| unit-test | テストディレクトリ | `testDir` |
| integration-test | テストディレクトリ | `testDir` |
| db-init | シードデータディレクトリ | `seedDir` |

#### 2.2 ワークフローファイルの検査

ci-test の `workflows` に定義されたワークフローが `.github/workflows/` に存在するか確認する。

```bash
ls .github/workflows/
```

#### 2.3 ビルドコマンドの検査

`skills_config.json` の `build` セクションに定義されたコマンドが実行可能か確認する。

- `./gradlew` → `gradlew` ファイルの存在確認
- `npm` / `npx` → `package.json` の存在確認
- `yarn` → `yarn.lock` の存在確認
- `pnpm` → `pnpm-lock.yaml` の存在確認
- `make` → `Makefile` の存在確認

### Phase 3: 診断結果の報告

検査結果を以下の形式で報告する。

```
## スキル設定診断結果

### グローバル設定 (.claude/skills/skills_config.json)

| 設定キー | 状態 | 現在の値 |
|----------|------|----------|
| branches.base | ✅ 設定済み | develop |
| build.fastCommand | ⚠️ 未設定 | （デフォルト: ./gradlew ...） |
| docs.bestPractices | ❌ 未設定（参照先ファイルも存在しない） | - |

### スキル別診断

#### ✅ create-issue（問題なし）
設定不要のスキル。

#### ⚠️ review
| 項目 | 状態 | 説明 |
|------|------|------|
| skill_config.json | ✅ 存在 | |
| excludePatterns | ✅ 設定済み | 4パターン |
| docs.bestPractices | ❌ ファイル不在 | `.claude/skills/practices/Best-Practices-Java.md` が存在しません |

**対応が必要:**
1. ベストプラクティスファイルを作成するか、`docs.bestPractices` を空配列 `[]` に設定してください
2. ベストプラクティスなしでもレビューは動作しますが、共通観点のみのレビューになります

#### ❌ ci-test
| 項目 | 状態 | 説明 |
|------|------|------|
| skill_config.json | ✅ 存在 | |
| workflows[0].file | ❌ ファイル不在 | `.github/workflows/api-test.yml` が存在しません |
| workflows[1].file | ❌ ファイル不在 | `.github/workflows/integration-test.yml` が存在しません |

**対応が必要:**
1. `.github/workflows/` にワークフローファイルを作成してください
2. または `skill_config.json` の `workflows` を実際のワークフローファイル名に更新してください

### サマリ

| 状態 | スキル数 |
|------|----------|
| ✅ 問題なし | 5 |
| ⚠️ 軽微な不足 | 3 |
| ❌ 要対応 | 2 |
```

### Phase 4: ユーザーヒアリング

診断結果に基づき、不足設定の解決に必要な情報をユーザーにヒアリングする。

**ヒアリングのルール:**
- 一度に聞く質問は 3-5 個まで。多すぎると負担になる
- 選択肢があるものは選択肢を提示する
- リポジトリから推測できるものは「これで合ってますか？」と確認する形にする

#### ヒアリング例

```
設定に不足があります。以下を教えてください:

1. **ビルドツール**: リポジトリに `package.json` があるので Node.js プロジェクトですね？
   - ビルドコマンド: `npm run build` でよいですか？
   - テストコマンド: `npm test` でよいですか？
   - フォーマッタ: prettier / eslint / biome のどれを使っていますか？

2. **テストフレームワーク**: Jest / Vitest / Mocha のどれですか？

3. **CI ワークフロー**: `.github/workflows/` に以下が見つかりました:
   - `test.yml` - これをテストワークフローとして登録しますか？
   - `lint.yml` - これもチェック対象に含めますか？
```

### Phase 5: リポジトリ調査と設定自動生成

ユーザーの回答とリポジトリの構造から設定を自動生成する。

#### 5.1 リポジトリ構造の調査

```bash
# プロジェクト構造の把握
ls -la
cat package.json 2>/dev/null || cat build.gradle 2>/dev/null || cat pom.xml 2>/dev/null || cat Makefile 2>/dev/null

# テストディレクトリの特定
# Node.js
ls __tests__/ test/ tests/ src/**/*.test.* src/**/*.spec.* 2>/dev/null

# Java
ls src/test/java/ 2>/dev/null

# Python
ls tests/ test/ 2>/dev/null

# CI ワークフローの特定
ls .github/workflows/ 2>/dev/null
```

#### 5.2 言語・フレームワークの自動検出

| 検出対象 | 判定方法 |
|----------|----------|
| Java/Gradle | `build.gradle` or `build.gradle.kts` の存在 |
| Java/Maven | `pom.xml` の存在 |
| Node.js/npm | `package.json` + `package-lock.json` |
| Node.js/yarn | `package.json` + `yarn.lock` |
| Node.js/pnpm | `package.json` + `pnpm-lock.yaml` |
| Python | `pyproject.toml` or `setup.py` or `requirements.txt` |
| Go | `go.mod` の存在 |
| Rust | `Cargo.toml` の存在 |

テストフレームワークの検出:

| 検出対象 | 判定方法 |
|----------|----------|
| JUnit 5 | `build.gradle` に `junit-jupiter` dependency |
| Jest | `package.json` の `devDependencies` に `jest` |
| Vitest | `package.json` の `devDependencies` に `vitest` |
| pytest | `pyproject.toml` の `[tool.pytest]` or `requirements` に `pytest` |
| Go test | `go.mod` の存在（標準ライブラリ） |

フォーマッタの検出:

| 検出対象 | 判定方法 |
|----------|----------|
| Spotless | `build.gradle` に `spotless` plugin |
| Prettier | `package.json` に `prettier` or `.prettierrc` の存在 |
| ESLint | `package.json` に `eslint` or `.eslintrc*` の存在 |
| Biome | `biome.json` の存在 |
| Black | `pyproject.toml` に `[tool.black]` |
| Ruff | `pyproject.toml` に `[tool.ruff]` |

#### 5.3 設定ファイルの生成

検出結果とユーザーの回答に基づき、設定ファイルを生成・更新する。

**生成対象:**

1. **skills_config.json**（未設定項目のみ追加）
   - `branches.base`: デフォルトブランチを `git remote show origin` から検出
   - `build.*`: 言語・ツールに応じたコマンドを設定
   - `docs.*`: 検出されたドキュメントパスを設定

2. **各スキルの skill_config.json**（未存在 or 未設定項目のみ）
   - unit-test: `language`, `testFramework`, `testDir`, `testFilePattern`, `testCommand`, `singleTestCommand`
   - integration-test: `language`, `testFramework`, `apiChangePatterns`
   - ci-test: `workflows`（`.github/workflows/` から検出）
   - review: `excludePatterns`（ビルド出力ディレクトリ等を自動検出）
   - pre-commit: `errorHints`（検出したツールに応じたヒントを設定）

#### 5.4 生成内容の確認

生成した設定をユーザーに提示し、確認を求める。

```
以下の設定を生成しました。内容を確認してください:

### skills_config.json（更新）
- branches.base: "main"（検出結果）
- build.test: "npm test"
- build.formatCheck: "npx prettier --check ."
- build.formatApply: "npx prettier --write ."

### unit-test/skill_config.json（新規作成）
- language: "typescript"
- testFramework: "jest"
- testDir: "__tests__"
- testFilePattern: "{ClassName}.test.ts"
- testCommand: "npm test"
- singleTestCommand: "npx jest {testClass}"

この内容で設定ファイルを作成しますか？
```

ユーザーの確認後、Write / Edit ツールで設定ファイルを作成・更新する。

### Phase 6: 再診断と結果報告

設定生成後、Phase 1-2 を再実行して問題が解消されたことを確認する。

```
## セットアップ完了

### 生成・更新したファイル
| ファイル | アクション |
|----------|-----------|
| .claude/skills/skills_config.json | 更新（3項目追加） |
| .claude/skills/unit-test/skill_config.json | 新規作成 |
| .claude/skills/ci-test/skill_config.json | 更新（workflows を修正） |

### 再診断結果
| 状態 | スキル数 |
|------|----------|
| ✅ 問題なし | 9 |
| ⚠️ 軽微な不足 | 1 |

⚠️ review: ベストプラクティスファイルが未作成です。
   プロジェクトのコーディング規約がある場合は `.claude/skills/practices/` に配置すると
   レビュー精度が向上します。
```

## 注意事項

- 既存の設定ファイルは上書きしない。未設定項目のみ追加する
- ユーザーの確認なしに設定ファイルを変更しない
- リポジトリ構造から推測した値は必ずユーザーに確認を取る
- M5 固有のスキル（db-init, m5-knowledge）はM5プロジェクト以外では診断対象外とする
