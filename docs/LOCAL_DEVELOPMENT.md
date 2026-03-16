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
src/config.rs       # YAML 設定パースのユニットテスト
src/comparator.rs   # 制約比較ロジックのユニットテスト
tests/
  integration_test.rs         # CLI 経由のインテグレーションテスト
  fixtures/
    register.html             # 登録フォーム (email, username, password)
    settings.html             # 設定フォーム (number, pattern, url, textarea, select)
    boundform.yml             # register.html に一致する設定
    boundform_mismatch.yml    # 意図的に不一致な設定
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
# サンプルアプリに対して validate
cargo run -- --config examples/sample-app/boundform.yml

# テストフィクスチャに対して validate
cargo run -- --config tests/fixtures/boundform.yml
```

実行結果の例:

```
[register] examples/sample-app/index.html
  username
    ✓ type = text
    ✓ required = true
    ✓ minlength = 3
    ✓ maxlength = 20
    ✓ pattern = [a-zA-Z0-9_]+
  email
    ✓ type = email
    ✓ required = true
  password
    ✓ type = password
    ✓ required = true
    ✓ minlength = 10
    ✓ maxlength = 128
  ...

1 page(s), 2 form(s), all 34 checks passed
```

### JSON 出力

```bash
cargo run -- --config examples/sample-app/boundform.yml --format json
```

### ローカルサーバーに対して実行

```yaml
# boundform.yml
pages:
  - url: "http://localhost:3000/register"
    forms:
      - index: 0
        fields:
          email:
            type: email
            required: true
```

```bash
cargo run -- --config boundform.yml
```

### Windows ホストで実行

Devcontainer 内でクロスコンパイル後、Windows 側の PowerShell/cmd から:

```powershell
.\target\x86_64-pc-windows-gnu\release\boundform.exe --config boundform.yml
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
│   ├── source.rs               # URL / ファイルから HTML 取得 (cookie/header 対応)
│   ├── parser.rs               # HTML パース、フォーム・フィールド抽出
│   ├── model.rs                # データ構造 (FormField, FormInfo, InputType)
│   ├── config.rs               # YAML 設定パース
│   ├── comparator.rs           # 期待値 vs 実際の制約比較
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
