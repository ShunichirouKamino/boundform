# Local Development Guide

boundform の開発環境セットアップからビルド、サンプルアプリでの動作確認、Windows 向けクロスコンパイルまでの手順を記載します。

## 前提条件

- [Docker](https://www.docker.com/)
- [VS Code](https://code.visualstudio.com/) + [Dev Containers 拡張](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

> Devcontainer を使わない場合は「[Devcontainer を使わない場合](#devcontainer-を使わない場合)」を参照してください。

## 1. 環境構築 (Devcontainer)

```bash
# VS Code でプロジェクトを開く
code .

# コマンドパレット → "Dev Containers: Reopen in Container"
```

コンテナ起動時に以下が自動でセットアップされます:

| ツール | 管理 | 用途 |
|---|---|---|
| Rust (latest stable) | mise | コンパイラ + cargo |
| Node.js (LTS) | mise | Claude Code CLI の実行環境 |
| Claude Code CLI | mise (npm) | AI アシスタント |
| gcc-mingw-w64-x86-64 | apt | Windows 向けクロスコンパイル |
| x86_64-pc-windows-gnu | rustup target | Windows ターゲット |

セットアップの確認:

```bash
rustc --version        # Rust コンパイラ
cargo --version        # パッケージマネージャ
mise ls                # mise で管理されている全ツール
rustup target list --installed  # x86_64-pc-windows-gnu が含まれること
```

## 2. ビルド

### Linux 向け (開発用)

```bash
# デバッグビルド (高速コンパイル、デバッグ情報あり)
cargo build

# リリースビルド (最適化あり)
cargo build --release
```

バイナリの出力先:

```
target/debug/boundform          # デバッグ
target/release/boundform        # リリース
```

### Windows 向け (クロスコンパイル)

Devcontainer 内で Windows 用の `.exe` をビルドできます:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

バイナリの出力先:

```
target/x86_64-pc-windows-gnu/release/boundform.exe
```

> `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER` は `mise.toml` で設定済みのため、追加の環境変数は不要です。

Windows ホスト側では、プロジェクトディレクトリ内の上記パスからそのまま `.exe` を実行できます (Devcontainer の `/workspaces` はホストにマウントされています)。

## 3. テスト

```bash
# 全テスト実行 (ユニットテスト + インテグレーションテスト)
cargo test

# 特定のテストのみ実行
cargo test test_parse_simple_form
cargo test parser::        # parser モジュールのテストのみ
cargo test validator::     # validator モジュールのテストのみ

# テスト出力を表示
cargo test -- --nocapture
```

### テスト構成

```
src/parser.rs       # HTML パース関連のユニットテスト
src/generator.rs    # テストケース生成のユニットテスト
src/validator.rs    # バリデーションロジックのユニットテスト
tests/
  integration_test.rs         # CLI 経由のインテグレーションテスト
  fixtures/
    register.html             # 登録フォーム (email, username, password)
    settings.html             # 設定フォーム (number, pattern, url, textarea, select)
    no_form.html              # フォームなし (エラーケース検証用)
```

## 4. Lint & Format

```bash
# Clippy (リンター)
cargo clippy

# フォーマットチェック (CI 向け)
cargo fmt -- --check

# 自動フォーマット
cargo fmt
```

## 5. サンプルアプリで動作確認

プロジェクトにサンプル HTML が含まれています。

### ローカルファイルで確認

```bash
# サンプルアプリ (登録フォーム + フィードバックフォーム)
cargo run -- check examples/sample-app/index.html

# テストフィクスチャ
cargo run -- check tests/fixtures/register.html
cargo run -- check tests/fixtures/settings.html
```

実行結果の例 (`examples/sample-app/index.html`):

```
[register form] 5 field(s) found

  username (type=text, required, minlength=3, maxlength=20, pattern)
    ✓ empty string → rejected (required)
    ✓ 2 chars → rejected (below minlength=3)
    ✓ 3 chars → accepted (at minlength boundary)
    ✓ 4 chars → accepted (above minlength)
    ✓ 19 chars → accepted (below maxlength)
    ✓ 20 chars → accepted (at maxlength boundary)
    ✓ 21 chars → rejected (above maxlength=20)
    ✓ non-matching string → rejected (pattern=[a-zA-Z0-9_]+)
    ✓ matching string → accepted (pattern=[a-zA-Z0-9_]+)

  email (type=email, required)
    ✓ empty string → rejected (required)
    ✓ invalid email (no @) → rejected (type=email)
    ✓ valid email → accepted

  password (type=password, required, minlength=10, maxlength=128)
    ✓ empty string → rejected (required)
    ✓ 9 chars → rejected (below minlength=10)
    ✓ 10 chars → accepted (at minlength boundary)
    ...

  age (type=number, min=13, max=150, step=1)
    ✓ 12 → rejected (below min=13)
    ✓ 13 → accepted (at min boundary)
    ✓ 150 → accepted (at max boundary)
    ✓ 151 → rejected (above max=150)
    ...

[feedback form] 5 field(s) found
  ...
```

### JSON 出力

```bash
cargo run -- check examples/sample-app/index.html --format json
```

```json
{
  "form_identifier": "register",
  "field_count": 5,
  "field_results": [
    {
      "field_name": "username",
      "constraint_summary": "type=text, required, minlength=3, maxlength=20, pattern",
      "results": [
        {
          "test_case": {
            "field_name": "username",
            "description": "empty string → rejected (required)",
            "test_value": "",
            "constraint": "required",
            "expected": "rejected"
          },
          "actual": "rejected",
          "passed": true
        }
      ]
    }
  ]
}
```

### ローカルサーバーに対して実行

自分のアプリケーションに対して boundform を実行する場合:

```bash
# Next.js / SvelteKit / Nuxt などの SSR アプリを起動
cd ~/my-nextjs-app
npm run dev
# → http://localhost:3000 で起動

# 別ターミナルで boundform を実行
cargo run -- check http://localhost:3000/register
cargo run -- check http://localhost:3000/login
cargo run -- check http://localhost:3000/settings
```

### Windows ホストで実行

Devcontainer 内でクロスコンパイル後、Windows 側の PowerShell/cmd から:

```powershell
# プロジェクトディレクトリ内から
.\target\x86_64-pc-windows-gnu\release\boundform.exe check .\examples\sample-app\index.html

# ローカルサーバーに対して
.\target\x86_64-pc-windows-gnu\release\boundform.exe check http://localhost:3000/register

# PATH の通った場所にコピーして使う場合
copy .\target\x86_64-pc-windows-gnu\release\boundform.exe $env:USERPROFILE\.local\bin\
boundform.exe check http://localhost:3000/register
```

## 6. 自分の HTML で試す

任意の HTML ファイルを作成して boundform に渡せます:

```html
<!-- my-form.html -->
<form name="my-form">
  <input type="text" name="name" required maxlength="100" />
  <input type="email" name="email" required />
  <input type="number" name="quantity" min="1" max="99" step="1" />
  <textarea name="note" minlength="10" maxlength="500"></textarea>
</form>
```

```bash
cargo run -- check my-form.html
```

## Devcontainer を使わない場合

Devcontainer を使わずにローカルマシンで直接開発する場合の手順です。

### 必要なツール

1. **Rust** — [rustup](https://rustup.rs/) でインストール
2. **システム依存パッケージ**

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install

# Windows (MSVC)
# Visual Studio Build Tools をインストール
```

3. **(任意) mise** — [mise.jdx.dev](https://mise.jdx.dev/) でインストール

```bash
# mise を使う場合: プロジェクトの mise.toml に従って自動セットアップ
mise trust && mise install
```

### Windows 向けクロスコンパイル (Linux/macOS から)

```bash
# ターゲット追加
rustup target add x86_64-pc-windows-gnu

# MinGW インストール
# Ubuntu/Debian:
sudo apt-get install gcc-mingw-w64-x86-64
# macOS:
brew install mingw-w64

# ビルド
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
  cargo build --release --target x86_64-pc-windows-gnu
```

## プロジェクト構成

```
boundform/
├── .devcontainer/
│   ├── Dockerfile              # ベースイメージ + apt 依存 (MinGW 含む)
│   ├── devcontainer.json       # VS Code 設定、拡張機能
│   └── post-create.sh          # mise install, rustup target 追加
├── src/
│   ├── main.rs                 # CLI エントリポイント (clap)
│   ├── error.rs                # カスタムエラー型 (thiserror)
│   ├── source.rs               # URL / ファイルから HTML 取得
│   ├── parser.rs               # HTML パース、フォーム・フィールド抽出
│   ├── model.rs                # データ構造 (FormField, BoundaryTestCase 等)
│   ├── generator.rs            # 境界値テストケース生成
│   ├── validator.rs            # HTML5 バリデーションルール評価
│   └── reporter.rs             # 出力フォーマット (Terminal / JSON)
├── tests/
│   ├── integration_test.rs     # CLI インテグレーションテスト
│   └── fixtures/               # テスト用 HTML ファイル
├── examples/
│   └── sample-app/
│       └── index.html          # 動作確認用サンプルフォーム
├── docs/
│   ├── DESIGN_DECISIONS.md     # 設計方針・背景
│   └── LOCAL_DEVELOPMENT.md    # このドキュメント
├── Cargo.toml
├── mise.toml                   # mise ツール定義 + 環境変数
├── CLAUDE.md                   # AI アシスタント向けプロジェクト説明
└── README.md
```
