# ADR-0005: HTML Attributes as the Validation Surface

## Status

Accepted

## Date

2026-03-16

## Context

boundform はSSRで返されるHTMLの制約属性（`required`, `min`, `max`, `minlength`, `maxlength`, `pattern`, `type`, `step`）を検証するツールである。しかし、モダンなフロントエンド開発では Zod などの JavaScript スキーマバリデーションライブラリが広く普及しており、以下のような状況が生まれている。

### Zod のみで制約を定義した場合

```tsx
// schema.ts
const schema = z.object({
  password: z.string().min(8).max(128),
});

// register/page.tsx
<form onSubmit={handleSubmit(onSubmit)}>
  <input {...register("password")} />   // HTML属性なし
</form>
```

SSRで返されるHTML:

```html
<input name="password" />
```

`required` も `minlength` も存在しない。Zodの制約はJSオブジェクトとしてのみ存在し、HTMLには一切反映されない。この状態では boundform が検証できるものがない。

### HTML属性を付与する価値

HTML5制約属性には、Zodでは代替できない独自の価値がある:

1. **プログレッシブエンハンスメント** — JSが読み込まれる前、またはJSが壊れた場合でもブラウザネイティブのバリデーションが動作する。Next.js の Server Actions はJS無しでのフォーム送信を前提としており、HTML属性がなければバリデーションなしで送信される。

2. **UXの即時性** — ブラウザネイティブバリデーションはフレームワークのレンダリングサイクルを経由しない。`required` 属性があれば submit 時に即座にブロックされ、Zodの検証結果を待つ必要がない。

3. **アクセシビリティ** — `required` 属性はスクリーンリーダーに「必須項目」として伝達される。Zodだけではこの情報がDOMに存在せず、支援技術に伝わらない。WCAG準拠の観点で重要。

4. **多層防御** — HTML属性（ブラウザ層）、Zodクライアント検証（JS層）、サーバー側検証（サーバー層）の3層で守ることがセキュリティのベストプラクティス。

### 二重管理の問題と解決策

「Zodとhtml属性の両方に制約を書くのは二重管理」という懸念に対して、conform のようなライブラリが解決策を提供している。

[conform](https://conform.guide/) は Zodスキーマを単一の情報源（Single Source of Truth）として、3つの層すべてに制約を展開する:

```
Zodスキーマ（1箇所に定義）
  ├→ HTML属性（ブラウザネイティブ、JS不要）  ← conform が自動生成
  ├→ クライアントJS（リッチなUX）             ← zodResolver が処理
  └→ サーバー側（改ざん防止）                 ← Server Actions で検証
```

conform を使えば、開発者はZodスキーマを1箇所書くだけで、HTML属性は自動的に付与される。二重管理にはならない。

## Decision

### boundform の守備範囲の明確化

boundform は **最終成果物（SSRで返されるHTML）に期待する制約属性が存在するか** を検証するツールと位置づける。

```
Zodスキーマ → (conform等) → SSR HTML → (boundform) → 制約チェック
                                          ↑
                           ここだけが守備範囲
```

boundform は:
- HTML属性の有無と値を検証する
- 制約がどのように実装されたか（手書き、conform、その他）には関与しない
- ZodスキーマやJSコードの解析は行わない

### 推奨スタック（Next.js + Zodの場合）

| レイヤー | ツール | 役割 |
|---|---|---|
| スキーマ定義 | Zod | Single Source of Truth |
| HTML属性付与 | conform | Zodスキーマ → HTML属性の自動変換 |
| フォーム管理 | react-hook-form + @hookform/resolvers/zod | クライアント側バリデーション |
| HTML制約検証 | boundform | HTML属性がYAML仕様通りか検証 |
| サーバー検証 | Zod (Server Actions) | 改ざん防止 |

### ワークフロー

```
1. 開発者: Zodスキーマを定義
2. conform: Zodスキーマ → HTML属性を自動生成
3. 開発者: boundform.yml に期待する制約を記載
4. CI: boundform でSSR HTMLを検証
5. 検出例: 「password フィールドに minlength=8 が期待されるが、HTMLに存在しない」
```

このワークフローにより、以下を検出できる:
- conform の設定漏れによりHTML属性が付与されていないケース
- デプロイ後にHTML属性が消えた制約ドリフト
- conform を使わないプロジェクトでのHTML属性の付け忘れ

## Consequences

### Positive

- **フレームワーク非依存** — boundform はHTML出力のみを見るため、Zod / conform / react-hook-form / どのライブラリを使っていても動作する
- **Zodとの共存が明確** — 競合ではなく補完関係。Zodは「何を検証するか」、boundform は「HTMLにそれが反映されているか」
- **conform 利用の後押し** — boundform をCIに入れることで、HTML属性を付与する動機が生まれる
- **アクセシビリティの向上** — HTML属性の存在を強制することで、間接的にWCAG準拠を促進

### Negative

- **Zod-onlyプロジェクトへのハードル** — conform等を導入しないプロジェクトでは、HTML属性を手書きする負担がある。ただしこれはboundformの問題ではなく、HTML5ベストプラクティスの問題
- **conform 以外のライブラリへの依存情報** — conform は Remix/Next.js エコシステムのライブラリであり、他のフレームワークには別のアプローチが必要

### ドキュメントへの反映

- README: Zod + conform との連携セクションを追加
- DESIGN_DECISIONS.md: HTML属性を前提とする設計判断と根拠を追記
