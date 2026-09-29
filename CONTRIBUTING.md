# 參與默契

謝謝你願意幫忙！回報問題、改一個錯字、加一個測試，都很有幫助。

> English speakers welcome — issues and PRs in English are fine. The docs are in Traditional Chinese because most users dictate in Mandarin.

## 幾種參與方式

- **回報問題**：用 [Issues](../../issues/new/choose) 的表單。附上 macOS 版本、在哪個 App 打字、選哪個潤飾程度，最有幫助。
- **聽錯的例子**：默契哪句中英夾雜聽錯了，把「你說的」和「它寫的」貼上來（不用附錄音）。這是改善辨識和整理最直接的材料。
- **建議功能**：從你遇到的情境講起。先看 README 的「接下來」。
- **程式碼**：小修正直接開 PR；比較大的改動，先開 issue 聊方向，免得白做。

## 開發環境

照 [BUILD.md](BUILD.md) 安裝工具、跑起來。架構和程式碼地圖在 [docs/架構.md](docs/架構.md)。

## 原則

改動前先看這幾條，它們是默契的產品承諾：

1. **聲音不離開電腦。** 辨識一律在本機。
2. **送出的東西看得到。** 任何送到網路的資料，都要出現在歷史紀錄的「這次送出了什麼」，並寫進 [docs/隱私.md](docs/隱私.md)。
3. **不丟稿。** 網路、潤稿、權限任何一步失敗，都要把原文交到使用者手上（貼上或複製）。
4. **不打擾。** 不搶焦點、不在錯的地方貼上、還原剪貼簿。
5. **不換使用者的用詞。** 整理只去贅字和改口，不替人潤飾風格（除非選了「潤飾」）。

## 程式碼慣例

- 我們的程式碼在 `src-tauri/src/yuyin/` 和 `src/yuyin/`。改到 Handy 原本的檔案時，在改動處加註解 `Yuyin fork:` 並說明原因，方便日後合併上游。規則見 [FORK.md](FORK.md)。
- Rust：`cargo fmt`；避免在正式路徑用 `unwrap()`；新行為要有單元測試。
- 前端：TypeScript 嚴格模式、React 函式元件、Tailwind。介面文字都放在 `src/i18n/locales/`（至少 `zh-TW` 和 `en`），ESLint 會擋寫死的字串。
- 提交訊息用 `feat:`、`fix:`、`docs:`、`refactor:`、`chore:` 開頭，內容寫**為什麼**，不只是做了什麼。

## 改到辨識或整理時

這兩塊的品質靠評測守住，不靠感覺：

- **整理的 prompt**（`src-tauri/src/yuyin/prompt.rs`）有逐字比對的測試（`testdata/prompt_v3_*.txt`）。改 prompt 要一併更新測試，並在 PR 說明用什麼句子驗證過。
- **邊說邊轉的切法**（`chunker.rs`）改動後，請說明長段的等待時間和錯字率有沒有變。
- 方法和目前的數字在 [docs/評測.md](docs/評測.md)。

## 送出 PR 前

```sh
cd src-tauri && cargo fmt && cargo test --release --lib && cd ..
bun x prettier --write . && bun run lint && bun x tsc --noEmit
```

PR 範本會提醒你寫驗證方式和隱私影響。用了 AI 工具也沒關係，說明一下你怎麼確認它寫的東西是對的。

## 行為準則

參與本專案即表示同意遵守 [行為準則](CODE_OF_CONDUCT.md)。
