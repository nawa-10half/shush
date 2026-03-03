# shush — アイデアメモ

## 発想の起点

- VibeCodingが普及する中で、AIエージェントが `git add . && git push` を誤実行するリスクが増大
- APIキー・シークレットの管理は既存の `.env` + `.gitignore` では本質的に解決できていない
- クラウド型シークレットマネージャーはセキュリティ意識の高い開発者には受け入れられにくい

## コアコンセプト

**「プロジェクトディレクトリに本物のキーが物理的に存在しない」設計**

```
プロジェクト内:   OPENAI_KEY=@vault:openai   ← pushしても無害
ローカル金庫:     openai → sk-abc123...       ← ~/.shush/ にのみ存在
```

## 差別化ポイント

1. **VibeCoding特化** — Claude / Cursor などのAIツールのhooksにネイティブ対応
2. **pushしても安全** — 防止だけでなく、設計レベルで安全
3. **開発者UX** — GitHubユーザー名だけでチーム共有できる
4. **完全ローカル** — サーバー不要、クラウドにキーは渡らない

## チーム共有方式

公開鍵ベースの非同期共有（D方式）を採用：

- 各開発者のGitHub SSH公開鍵を利用（新たな鍵管理不要）
- 暗号化ブロブをSlack・メール・任意の経路で共有可能
- サーバー不要、オフライン対応

## 技術スタック

| 項目 | 選定 | 理由 |
|------|------|------|
| 言語 | Rust | メモリ安全、バイナリ配布、セキュリティ信頼性 |
| 暗号化 | age | モダン、SSH鍵互換、シンプル |
| ストレージ | SQLite + SQLCipher | 実績あり、暗号化DB |
| 鍵解錠 | macOS Keychain / Touch ID | UX最優先 |
| フォールバック | Argon2id（マスターパスワード） | Linux対応 |

## MVP スコープ（v0.1）

- [ ] `shush add <name> <value>` — シークレット追加
- [ ] `shush list` — 一覧表示（値は表示しない）
- [ ] `shush delete <name>` — 削除
- [ ] `shush run <command>` — 環境変数注入して実行
- [ ] `shush scan` — 生のシークレット値を検出
- [ ] `shush install-hooks` — git + Claude hooksを自動設定

## scan機能の設計詳細

`shush scan` はパターンマッチ（`sk-`で始まる等）ではなく、**金庫に登録された実際の値と照合**するのが差別化ポイント。

```
git push 実行
    ↓
pre-push hook が shush scan を実行
    ↓
金庫の実値とコード全体を照合
    ↓
検出なし → そのままpush ✅
検出あり → 警告 & ブロック ❌

⚠️  shush: Secret detected before push!
    File: src/api.ts:12
    Key:  OPENAI_KEY (matches vault value)

    Run `shush run` to inject secrets safely.
    Commit aborted.
```

- `.env` ファイルだけでなく、**ソースコードへの直書き**（VibeCodingでAIがやりがち）も検出できる
- `shush install-hooks` でpre-push hookとClaude hooksを一括セットアップ
- **登録していないキーは検出できない**という前提はある（自分の金庫に登録したキーを守るツール）

## v2以降の候補機能

- チーム共有（`shush share / receive`）
- `.vault.team` によるgit経由のチーム同期
- スコープ付きトークン（プロジェクト限定、有効期限）
- Cursor / GitHub Copilot hooks対応

## プロダクト形態

- **OSS**（MIT License）
- マネタイズなし（まず使われるものを作る）
- 配布: `cargo install shush` / Homebrew tap

## 競合との比較

| ツール | ローカル | 開発者UX | AI統合 | チーム共有 |
|--------|---------|---------|--------|---------|
| KeePass | ✅ | ❌ | ❌ | △ |
| Doppler | ❌ | ✅ | △ | ✅ |
| HashiCorp Vault | △ | ❌ | ❌ | ✅ |
| **shush** | ✅ | ✅ | ✅ | ✅ |

## 次のステップ候補

- [ ] Rustプロジェクトの雛形作成（`cargo init`）
- [ ] ageライブラリの調査・PoC
- [ ] SQLCipherのRust bindings確認
- [ ] CLIコマンド設計の詳細化
