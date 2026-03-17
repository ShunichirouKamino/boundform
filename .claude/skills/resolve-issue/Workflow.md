# ワークフロー（状態遷移管理）

M5 におけるワークフローの仕組みを解説する。ワークフローは承認フロー・削除フローなどの状態遷移を管理するオートマトンパターンで実装されている。

## 1. 概念モデル

### 1.1 ワークフローの構成要素

```
Workflow（ワークフロー定義）
  ├── WorkflowNode（ノード＝ステータス定義）
  │     例: UNAUTHORIZED → AUTHORIZED → CANCELLED
  │
  ├── WorkflowSetting（遷移ルール）
  │     CurrentNode + Function → NextNode
  │     ├── WorkflowSettingUser（ユーザ制限）
  │     ├── WorkflowSettingOrganization（組織制限）
  │     ├── WorkflowSettingOrganizationLevel（階層制限）
  │     ├── WorkflowFunctionSetting（自動実行ファンクション）
  │     ├── WorkflowConsensusSetting（合議設定）
  │     └── WorkflowTermSetting（条件分岐設定）
  │
  ├── WorkflowCurrentNode（ランタイム状態）
  │     対象エンティティの現在のノードを追跡
  │
  ├── WorkflowHistory（監査証跡）
  │     全遷移の不変記録
  │
  └── WorkflowOrganizationMng（ドメイン割当）
        ワークフローに対象ドメインを紐付け
```

### 1.2 基本フロー

```
[UNAUTHORIZED] --(承認ファンクション)--> [AUTHORIZED]
      |                                      |
      +----(キャンセルファンクション)---> [CANCELLED]
```

- **ノード**: ワークフロー上の状態（ステータス）。1 つが初期ノード（`initial_node_flag = true`）
- **ファンクション**: ノード間の遷移をトリガーするアクション（`functions` テーブルで管理）
- **遷移ルール**: `WorkflowSetting` で `CurrentNode + Function → NextNode` を定義

### 1.3 ワークフローの種別

`WorkflowLabel` は `WorkflowPrefix` + `WorkflowType` で構成される。

| WorkflowType    | 説明     | ラベル例                                      |
|-----------------|--------|---------------------------------------------|
| `APPROVAL_FLOW` | 承認フロー  | `commodity-physical-trade-approval-flow`    |
| `DELETE_FLOW`   | 削除フロー  | `commodity-physical-trade-delete-flow`      |

`WorkflowPrefix` は業務エンティティの分類を示す（例: `commodity-physical-trade`）。

## 2. ER 構造

### 2.1 テーブル一覧

マイグレーション: `db/postgre/migration/V1_0_07__create_workflow.sql`

#### マスタテーブル

| テーブル                                | 説明           | ユニーク制約                                                     |
|-------------------------------------|--------------|--------------------------------------------------------------|
| `workflow`                          | ワークフロー定義     | `(service_organization_id, workflow_label)`                   |
| `workflow_node`                     | ノード（ステータス）定義 | `(workflow_id, workflow_node_label)`                          |
| `workflow_setting`                  | 遷移ルール       | `(current_node_id, function_id, workflow_id)`                |
| `workflow_function_setting`         | 自動実行ファンクション  | `(workflow_setting_id)`                                      |
| `workflow_consensus_setting`        | 合議設定         | `(workflow_consensus_setting_code)`                          |
| `workflow_term_setting`             | 条件分岐設定       | `(workflow_formula_code)`                                    |

#### ランタイムテーブル

| テーブル                       | 説明        | ユニーク制約                                                        |
|----------------------------|-----------|---------------------------------------------------------------|
| `workflow_current_node`    | 現在のノード状態  | `(workflow_id, target_domain, workflow_entity_id)`            |
| `workflow_history`         | 遷移履歴（不変）  | `(workflow_id, target_domain, workflow_entity_id, target_version)` |
| `workflow_consensus_state` | 合議承認状態    | `(workflow_consensus_setting_id, workflow_entity_id)`         |

#### 連関テーブル（アクセス制御）

| テーブル                                     | 説明            | PK                                          |
|------------------------------------------|---------------|---------------------------------------------|
| `workflow_setting_user`                  | ユーザ制限         | `(workflow_setting_id, user_id)`            |
| `workflow_setting_organization`          | 組織制限          | `(workflow_setting_id, organization_id)`    |
| `workflow_setting_organization_level`    | 組織階層制限        | `(workflow_setting_id, organization_level_id)` |
| `workflow_organization_mng`              | ドメイン割当        | `(workflow_id, organization_mng_id)`        |

### 2.2 主要カラム

#### `workflow`

| カラム                       | 型                | 説明                                |
|---------------------------|------------------|-----------------------------------|
| `id`                      | bigint PK        | サロゲートキー                           |
| `workflow_label`          | varchar(50)      | ワークフローラベル（`{prefix}-{type}`形式）    |
| `service_organization_id` | bigint FK        | サービス組織への参照                        |
| `workflow_label_name`     | varchar(50)      | 表示名                               |
| `description`             | varchar(256)     | 説明                                |
| `additional_info`         | jsonb            | 個社別拡張情報                           |

#### `workflow_node`

| カラム                       | 型                | 説明                                |
|---------------------------|------------------|-----------------------------------|
| `id`                      | bigint PK        | サロゲートキー                           |
| `workflow_id`             | bigint FK        | ワークフローへの参照                        |
| `workflow_node_label`     | varchar(30)      | ノードラベル（例: `UNAUTHORIZED`）         |
| `workflow_node_label_name`| varchar(50)      | ノード表示名                            |
| `initial_node_flag`       | boolean          | 初期ノードフラグ（1 ワークフローに 1 つ）          |

テンプレートラベル: `INITIAL`, `UNAUTHORIZED`, `AUTHORIZED`, `NOTNECESSARY`, `CANCELLED`

#### `workflow_setting`

| カラム               | 型           | 説明                          |
|-------------------|-----------|-----------------------------|
| `id`              | bigint PK | サロゲートキー                     |
| `workflow_id`     | bigint FK | ワークフローへの参照                  |
| `current_node_id` | bigint FK | 現在ノード（`workflow_node.id`）   |
| `function_id`     | bigint FK | 実行ファンクション（`functions.id`）   |
| `next_node_id`    | bigint FK | 遷移先ノード（`workflow_node.id`）  |
| `user_group_id`   | bigint FK | ユーザグループ（Deprecated）         |

#### `workflow_current_node`

| カラム                  | 型              | 説明                      |
|----------------------|----------------|-------------------------|
| `id`                 | bigint PK      | サロゲートキー                 |
| `workflow_id`        | bigint FK      | ワークフローへの参照              |
| `target_domain`      | varchar(63)    | 対象ドメイン（例: `market_price`）|
| `workflow_entity_id` | bigint         | 対象エンティティの PK            |
| `current_node_id`    | bigint FK      | 現在のノード                  |
| `function_id`        | bigint FK      | 最後に実行されたファンクション         |
| `organization_id`    | bigint FK      | 申請者の所属組織                |
| `version`            | bigint         | 楽観排他制御用バージョン            |

#### `workflow_history`

| カラム                  | 型           | 説明                    |
|----------------------|-------------|----------------------|
| `workflow_id`        | bigint FK   | ワークフローへの参照            |
| `target_domain`      | varchar(63) | 対象ドメイン               |
| `workflow_entity_id` | bigint      | 対象エンティティの PK          |
| `target_version`     | bigint      | 遷移時のエンティティバージョン       |
| `target_data`        | jsonb       | 遷移時のエンティティデータスナップショット |
| `function_id`        | bigint FK   | 実行されたファンクション          |
| `current_node_id`    | bigint FK   | 遷移後のノード              |

## 3. ドメインモデル

### 3.1 ValueObject

| クラス                        | ベース型                  | 説明                                      |
|----------------------------|-----------------------|-----------------------------------------|
| `WorkflowLabel`            | `ComparableStringBase`| ワークフローラベル（`{prefix}-{type}`形式、max 100 文字）|
| `WorkflowNodeLabel`        | `ComparableStringBase`| ノードラベル（ASCII、max 50 文字）                 |
| `WorkflowEntityId`         | `ComparableStringBase`| 対象エンティティ ID（1-10 桁数値文字列）                |
| `WorkflowEntityVersion`    | `ComparableStringBase`| ワークフローバージョン（1-10 桁数値文字列）                |
| `WorkflowPrefix`           | `ComparableStringBase`| ワークフローグループ接頭辞（ASCII）                    |
| `WorkflowApprovalOrder`    | `CodeEnum`            | 承認順序（FIRST〜TENTH）                       |
| `WorkflowApprovalState`    | `CodeEnum`            | 承認状態（APPROVAL のみ）                       |
| `WorkflowTargetApprover`   | `record`              | 承認者（`sysUserCode` or `userGroupCode`）   |
| `WorkflowFormula`          | `record`              | 条件式（`value`, `formula`, `result`, ユーザ/グループリスト）|
| `WorkflowFunctionSettingCondition` | `ComparableStringBase` | ファンクション設定条件（`AND`）       |

#### WorkflowLabel の構造

```java
// WorkflowPrefix + WorkflowType で構成
WorkflowLabel.of(WorkflowPrefix.of("commodity-physical-trade"), WorkflowType.APPROVAL_FLOW)
// → "commodity-physical-trade-approval-flow"

// prefix/type の抽出
label.getPrefix()  // → WorkflowPrefix("commodity-physical-trade")
label.getType()    // → WorkflowType.APPROVAL_FLOW
```

#### WorkflowEntityId の変換

```java
WorkflowEntityId.numOf(123L)     // 数値から生成
wfEntityId.toNumber()            // long に変換
wfEntityId.toSurrogateKey()      // SurrogateKey に変換
```

#### WorkflowFormula の構造（JSONB）

```json
{
  "value": "TRUE",
  "formula": "equal",
  "result": true,
  "userCode": ["WFTEST000001"],
  "userGroupCode": ["UG0001"]
}
```

formula の種類: `equal`, `like`, `empty`, `exclusiveMinimum`

### 3.2 EntityId（複合キー）

| クラス                     | 構成要素                                                                      |
|-------------------------|---------------------------------------------------------------------------|
| `WorkflowId`            | `ServiceCode` + `SysOrganizationCode` + `WorkflowLabel`                  |
| `WorkflowCurrentNodeId` | `ServiceCode` + `SysOrganizationCode` + `WorkflowLabel` + `TargetDomainDef` + `WorkflowEntityId` |
| `WorkflowNodeId`        | `SurrogateKey`（ノードの ID）                                                   |
| `WorkflowHistoryId`     | `ServiceCode` + `SysOrganizationCode` + `WorkflowLabel` + `WorkflowEntityId` + `WorkflowEntityVersion` |
| `WorkflowSettingId`     | `ServiceCode` + `SysOrganizationCode` + `WorkflowLabel` + `WorkflowNodeLabel` + `FunctionCode` |

### 3.3 Entity インタフェース

パッケージ: `m5-domain/.../domain/workflow/entity/`

| エンティティ                    | テーブル                         | 説明                  |
|--------------------------|------------------------------|---------------------|
| `Workflow`               | `workflow`                   | ワークフロー定義マスタ         |
| `WorkflowNode`           | `workflow_node`              | ノード（ステータス）定義        |
| `WorkflowSetting`        | `workflow_setting`           | 遷移ルール + アクセス制御情報    |
| `WorkflowCurrentNode`    | `workflow_current_node`      | ランタイム状態追跡           |
| `WorkflowHistory`        | `workflow_history`           | 監査証跡（不変）            |
| `WorkflowFunctionSetting`| `workflow_function_setting`  | 自動実行ファンクション         |
| `WorkflowConsensusSetting`| `workflow_consensus_setting`| 合議設定（Deprecated）    |
| `WorkflowConsensusState` | `workflow_consensus_state`   | 合議承認状態              |
| `WorkflowTermSetting`    | `workflow_term_setting`      | 条件分岐設定              |

### 3.4 Repository インタフェース

パッケージ: `m5-domain/.../domain/workflow/repository/`

| Repository                         | 主要メソッド                                          |
|------------------------------------|-------------------------------------------------|
| `WorkflowRepository`               | `get(WorkflowId)`, `selectByFunction(...)`      |
| `WorkflowNodeRepository`           | `getInitialNode(WorkflowId)`                    |
| `WorkflowSettingRepository`        | `selectByPointer(...)`, `selectSettings(...)`   |
| `WorkflowCurrentNodeRepository`    | `get(...)`, `select(...)`, `rollback(...)`      |
| `WorkflowHistoryRepository`        | `selectWorkflowHistories(...)`                  |
| `WorkflowFunctionSettingRepository`| `select(WorkflowNodeId)`                        |

## 4. 状態遷移の仕組み

### 4.1 遷移処理の流れ

```
1. クライアントが PUT /workflow/{workflowLabel} を呼び出す
2. WorkflowExecuteResourceV2 がリクエストを受信
3. ファンクション認可チェック（authorizationManager.hasFunctionRole）
4. WorkflowManagerV2.moveNextNode() を呼び出す
5. WorkflowChainedExecuter が以下を実行:
   a. WorkflowCurrentNode を取得（楽観排他チェック）
   b. WorkflowSetting から遷移ルールを検索
   c. CurrentNode + Function → NextNode を決定
   d. WorkflowCurrentNode を更新（current_node_id を nextNode に変更）
   e. WorkflowHistory に遷移記録を追加
   f. WorkflowFunctionSetting があれば自動ファンクションを連鎖実行
```

### 4.2 V2 API の 2 つの実行モード

#### マルチ実行（`PUT /workflow`）

業務操作の副作用としてワークフローを遷移させる。対象エンティティに紐づく全ワークフローを一括遷移。

```java
// WorkflowManagerV2.moveAllNode()
// 同一 workflowEntityId に紐づく全 WorkflowCurrentNode を取得し、
// 指定 functionCode で遷移可能なもの全てを遷移
```

用途: 取引登録時に承認フロー・削除フローの両方を初期化する場合など。

#### シングル実行（`PUT /workflow/{workflowLabel}`）

ワークフロー一覧画面から特定のワークフローを遷移させる。楽観排他制御あり。

```java
// WorkflowManagerV2.moveNextNode()
// 指定 WorkflowCurrentNodeId のワークフローのみ遷移
// entityVersion による楽観排他を実施
```

### 4.3 自動ファンクション連鎖（WorkflowFunctionSetting）

遷移完了後、`WorkflowFunctionSetting` に定義された自動ファンクションを連鎖的に実行する。

```
条件: condition = "AND"
動作: 遷移先ノードに WorkflowFunctionSetting があり、
      全ての条件（ユーザ/組織/階層の制限）が満たされている場合、
      execute_function_code のファンクションを自動実行する
```

例: 全員が承認完了した後に自動で「最終承認」ファンクションを実行

### 4.4 合議（コンセンサス）承認

複数承認者による段階的な承認フロー。

```
WorkflowConsensusSetting:
  FIRST  → ユーザ A が承認
  SECOND → ユーザグループ UG0001 の誰かが承認
  THIRD  → ユーザ B が承認
```

- `WorkflowConsensusState` で各承認者の承認状態を追跡
- 全段階の承認が完了すると、WorkflowFunctionSetting の条件が満たされ自動遷移

### 4.5 条件分岐（WorkflowTermSetting）

エンティティのカラム値に基づく条件分岐。

```json
// 例: 金額が 10000 を超える場合のみ特定の承認者に割り当て
{
  "formula": "exclusiveMinimum",
  "value": "10000",
  "result": true,
  "userCode": ["MANAGER001"]
}
```

- `workflow_target_entity` + `workflow_target_column` で評価対象を指定
- `workflow_formula` の条件式で分岐判定
- 条件に合致した場合、指定ユーザ/グループに遷移権限を付与

## 5. アクセス制御

### 5.1 遷移権限の 3 レイヤー

各 `WorkflowSetting` に対し、以下の連関テーブルで実行可能なユーザを制限する。

| レイヤー                                  | テーブル                                  | 制御単位        |
|---------------------------------------|---------------------------------------|-------------|
| ユーザ制限                                 | `workflow_setting_user`               | 個別ユーザ       |
| 組織制限                                  | `workflow_setting_organization`       | 組織に所属する全ユーザ |
| 組織階層制限                                | `workflow_setting_organization_level` | 階層レベルに該当する全ユーザ |

全レイヤーが空の場合は制限なし（全ユーザ実行可能）。

### 5.2 重複実行防止

`WorkflowSetting` エンティティの `sysUserCodes` / `sysOrganizationCodes` / `organizationLevelCodes` を使い、既に実行済みのユーザ・組織・階層レベルを追跡し、同一ノードでの重複実行を防止する。

### 5.3 Inspector との関係

ワークフロードメインの `organization_mng` 設定:

| ドメイン       | read         | write     |
|------------|--------------|-----------|
| `WORKFLOW` | `LOWER`      | `LOWER`   |

→ ワークフローの参照・更新は自組織 + 下位組織のデータのみ。

## 6. API エンドポイント

### 6.1 ワークフロー実行 API（V2）

サービス: `workflow`（ポート: 9106）

| メソッド | パス                        | 説明                        |
|------|---------------------------|---------------------------|
| GET  | `/workflow`               | ユーザが実行可能なカレントノード一覧を取得     |
| GET  | `/workflow/history`       | 対象エンティティのワークフロー履歴を取得      |
| PUT  | `/workflow`               | 全対象ワークフローを一括遷移（マルチ実行）     |
| PUT  | `/workflow/{workflowLabel}` | 特定ワークフローを遷移（シングル実行・排他あり） |

### 6.2 ワークフロー定義 API（V2）

| メソッド   | パス                                  | 説明                    |
|--------|-------------------------------------|-----------------------|
| GET    | `/workflow-define`                  | ワークフロー定義一覧を取得         |
| GET    | `/workflow-define/{workflowLabel}`  | ワークフロー定義の詳細（ノード・遷移ルール全体）を取得 |
| POST   | `/workflow-define`                  | ワークフロー定義を登録           |
| DELETE | `/workflow-define/{workflowLabel}`  | ワークフロー定義を論理削除         |

### 6.3 リクエスト/レスポンスモデル

#### GET `/workflow` のクエリパラメータ

| パラメータ                 | 必須  | 説明                    |
|-----------------------|-----|--------------------------------------|
| `serviceCode`         | Yes | サービスコード               |
| `sysOrganizationCode` | Yes | 組織コード                 |
| `sysUserCode`         | No  | フィルタ対象ユーザ             |
| `functionCode`        | No  | 実行可能ファンクション（複数指定可）    |
| `targetDomain`        | No  | 対象ドメイン                |
| `workflowEntityId`    | No  | 対象エンティティ ID           |
| `isActive`            | No  | 未完了のみ（default: `true`）|

#### PUT `/workflow` のリクエストボディ（`WorkflowMultiPutModelV2`）

```json
{
  "serviceCode": "workflow",
  "sysOrganizationCode": "ORG000000001",
  "targetDomain": "market_price",
  "workflowEntityId": "123",
  "functionCode": "APPROVE",
  "contents": { /* エンティティの JSON スナップショット */ },
  "additionalInfo": { /* 任意の拡張情報 */ }
}
```

#### PUT `/workflow/{workflowLabel}` のリクエストボディ（`WorkflowSinglePutModelV2`）

```json
{
  "serviceCode": "workflow",
  "sysOrganizationCode": "ORG000000001",
  "targetDomain": "market_price",
  "workflowEntityId": "123",
  "workflowLabel": "commodity-physical-trade-approval-flow",
  "functionCode": "APPROVE",
  "entityVersion": 1,
  "contents": { /* 任意 */ },
  "additionalInfo": { /* 任意 */ }
}
```

#### レスポンスモデル（`WorkflowCurrentNodeRetModelV2`）

```json
{
  "serviceCode": "workflow",
  "sysOrganizationCode": "ORG000000001",
  "workflowLabel": "commodity-physical-trade-approval-flow",
  "targetDomain": "market_price",
  "workflowEntityId": "123",
  "workflowNodeLabel": "AUTHORIZED",
  "workflowNodeLabelName": "承認済み",
  "nextFunctions": [
    {
      "functionCode": "CANCEL",
      "nextWorkflowNodeLabel": "CANCELLED",
      "nextWorkflowNodeLabelName": "キャンセル"
    }
  ],
  "sysFields": { /* 監査情報 */ }
}
```

## 7. システムレイヤー（WorkflowManager）

### 7.1 V1 と V2 の違い

| 項目             | V1（WorkflowManager）                | V2（WorkflowManagerV2）                     |
|----------------|-------------------------------------|-------------------------------------------|
| 遷移実行           | `moveNextNode(WorkflowModel)`       | `moveNextNode(WorkflowCurrentNodeId, ...)` / `moveAllNode(...)` |
| TargetDomain   | 未対応                                 | 対応                                        |
| 自動ファンクション連鎖   | 未対応                                 | `WorkflowChainedExecuter` で対応              |
| 楽観排他           | `WorkflowEntityVersion` ベース         | `EntityVersion` ベース（Optional）             |
| 合議承認           | `updateConsensusStatus`              | `WorkflowChainedExecuter` 内で処理            |
| ロールバック         | `rollback(WorkflowModel)`           | 未対応（V1 のみ）                                |

### 7.2 WorkflowManagerV2 の主要メソッド

```java
// ワークフローマスタ取得
Optional<Workflow> getWorkflow(WorkflowId id, TargetDomainDef targetDomain)

// カレントノード取得
Optional<WorkflowCurrentNode> getCurrentNode(WorkflowCurrentNodeId id)

// ユーザが実行可能なカレントノード + 遷移ルール一覧
List<Pair<WorkflowCurrentNode, List<WorkflowSetting>>> getCurrentNodesAndSettings(
    ServiceCode, SysOrganizationCode, SysUserCode,
    List<FunctionCode>, Optional<TargetDomainDef>,
    Optional<WorkflowEntityId>, Optional<SysUserCode> authorCode, boolean isActive)

// ワークフロー履歴取得
List<WorkflowHistory> getHistories(
    ServiceCode, SysOrganizationCode, TargetDomainDef,
    WorkflowEntityId, FunctionCode)

// ファンクション設定取得
List<WorkflowFunctionSetting> getFunctionSettings(WorkflowNodeId)

// 一括遷移（業務操作の副作用）
List<WorkflowCurrentNode> moveAllNode(
    ServiceCode, SysOrganizationCode, TargetDomainDef,
    WorkflowEntityId, FunctionCode, SysUserCode,
    M5EntitySysFields, Optional<JsonNode> contents)

// 単一遷移（ワークフロー画面から）
Optional<WorkflowCurrentNode> moveNextNode(
    WorkflowCurrentNodeId, FunctionCode,
    Optional<EntityVersion>, SysUserCode,
    M5EntitySysFields, Optional<JsonNode> contents)
```

## 8. 他ドメインとの連携

### 8.1 ユーザドメインとの連携

`UserSequencialFlowResoruce` がワークフローと連携:
- ユーザ登録時にワークフローカレントノードを作成
- `WorkflowCurrentNodeRepository` を注入して使用

### 8.2 ファンクション（functions テーブル）

ワークフローの遷移トリガーとなるファンクションは `functions` テーブルで管理される。`WorkflowSetting.function_id` が外部キーで参照する。ファンクション認可は `AuthorizationManager.hasFunctionRole()` で確認。

## 9. シードデータ

| ファイル                                             | 内容                              |
|--------------------------------------------------|---------------------------------|
| `db/postgre/sql/default/701_workflow.sql`        | ワークフロー定義（承認フロー・削除フローなど 51 件）    |
| `db/postgre/sql/default/702_workflow_node.sql`   | ノード定義（各ワークフローのステータス）            |
| `db/postgre/sql/default/703_workflow_setting.sql`| 遷移ルール定義                         |
| `db/postgre/sql/default/706_workflow_consensus_setting.sql` | 合議設定（承認順序と承認者）       |
| `db/postgre/sql/default/707_workflow_term_setting.sql` | 条件分岐設定（44 件）              |
| `db/postgre/sql/default/708_workflow_consensus_state.sql` | 合議承認状態（7 件）              |

## 10. 関連ソースファイル

### ドメイン層

| ファイル                                                                    | 説明             |
|-------------------------------------------------------------------------|----------------|
| `m5-domain/.../domain/workflow/entity/*.java`                           | Entity インタフェース |
| `m5-domain/.../domain/workflow/value/*.java`                            | ValueObject    |
| `m5-domain/.../domain/workflow/id/*.java`                               | EntityId（複合キー）|
| `m5-domain/.../domain/workflow/repository/*.java`                       | Repository インタフェース |

### リポジトリ層

| ファイル                                                                    | 説明                |
|-------------------------------------------------------------------------|-------------------|
| `m5-repository-doma/.../doma2/workflow/entity/*.java`                   | Doma2 Entity 実装   |
| `m5-repository-doma/.../doma2/workflow/dao/*.java`                      | Doma DAO          |
| `m5-repository-doma/.../doma2/workflow/repository/*.java`               | Repository 実装     |

### システム層

| ファイル                                                                    | 説明                   |
|-------------------------------------------------------------------------|----------------------|
| `m5-sys/.../sys/workflow/WorkflowManager.java`                          | V1 Manager インタフェース  |
| `m5-sys/.../sys/workflow/WorkflowManagerV2.java`                        | V2 Manager インタフェース  |
| `m5-sys/.../sys/workflow/impl/M5WorkflowManager.java`                  | V1 Manager 実装        |
| `m5-sys/.../sys/workflow/impl/M5WorkflowManagerV2.java`                | V2 Manager 実装        |
| `m5-sys/.../sys/workflow/model/*.java`                                  | リクエスト/レスポンスモデル      |

### API 層

| ファイル                                                                    | 説明                  |
|-------------------------------------------------------------------------|---------------------|
| `m5-api-workflow/.../resource/workflow/WorkflowService.java`            | ルートサービス（ルーティング）     |
| `m5-api-workflow/.../resource/workflow/WorkflowResource.java`           | V1 Resource         |
| `m5-api-workflow/.../resource/workflow/WorkflowExecuteResourceV2.java`  | V2 実行 Resource      |
| `m5-api-workflow/.../resource/workflow/WorkflowDefineResourceV2.java`   | V2 定義 Resource      |

### DB

| ファイル                                                 | 説明          |
|------------------------------------------------------|-------------|
| `db/postgre/migration/V1_0_07__create_workflow.sql`  | テーブル定義・制約・コメント |
| `db/postgre/sql/default/70x_workflow*.sql`           | シードデータ      |

## 11. 実装時の注意点

- ワークフロー定義の POST では、初期ノードが正確に 1 つ、非初期ノードが 1 つ以上であることをバリデーションする
- `WorkflowFunctionSetting` の `execute_function_code` に定義されたファンクションは手動実行不可（自動実行専用）
- `WorkflowCurrentNode` のユニーク制約 `(workflow_id, target_domain, workflow_entity_id)` により、1 エンティティにつき 1 ワークフロー 1 ドメインで 1 カレントノードのみ
- 楽観排他制御は V2 シングル実行時のみ。マルチ実行時はバージョンチェックなし
- `WorkflowHistory` は不変レコード。遷移ごとに `target_version` がインクリメントされ、ユニーク制約で重複を防止
- 合議設定（`WorkflowConsensusSetting`）は Deprecated だが、`WorkflowConsensusState` は引き続き使用されている
- 条件分岐（`WorkflowTermSetting`）の `workflow_formula` は JSONB 型。formula の種類は `equal`, `like`, `empty`, `exclusiveMinimum`
