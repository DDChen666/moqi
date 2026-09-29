## 改了什麼、為什麼

<!-- 從使用者遇到的情況講起。修 bug 的話，附上對應的 issue：Fixes #123 -->

## 怎麼驗證

<!-- 跑了哪些測試、手動試了哪些情境（在哪個 App、哪個潤飾程度）。改到畫面的話附截圖。 -->

- [ ] `cargo test --release --lib`（在 `src-tauri/`）
- [ ] `bun run lint`、`bun x prettier --check .`、`cargo fmt --check`
- [ ] 改到 Handy 原本的檔案時，改動處標了 `Yuyin fork:`（見 [FORK.md](../FORK.md)）

## 隱私

- [ ] 沒有多送出任何資料；或是多送了，寫在上面並更新了 [docs/隱私.md](../docs/隱私.md)

## AI 協助

<!-- 用了 AI 工具的話簡單說明（例如「Claude Code 寫初稿，我逐行讀過並測試」）。沒用就刪掉這段。 -->
