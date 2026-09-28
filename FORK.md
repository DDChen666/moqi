# 關於這個分支：默契 Moqi

> 產品名稱是「默契」（英文 Moqi）。程式碼裡的 `yuyin`、註解裡的 `Yuyin fork:` 是早期的內部代號，保留不改。

這個程式是從 [Handy](https://github.com/cjpais/Handy) 分出來的（作者 CJ Pais，MIT 授權）。
分出的版本：upstream `main` 的 `29bd2c0`（2026-09-28，Handy 0.9.7）。

## 為什麼從 Handy 改，不從頭寫

Handy 已經做好 Mac 上最難的部分：
- 抓得到 Fn、右 Option 的全域快捷鍵
- 不搶焦點的懸浮窗
- 按住說話、切換的狀態機
- transcribe.cpp 加 Qwen3-ASR
- 靜音偵測
- 歷史紀錄
- 權限引導

我們要加的是產品層的東西：情境判斷、焦點檢查、依 App 調整貼上方式、潤飾程度、失敗後一鍵重貼等等。
評估過程在 `../設計文件/產品定義_v0.md`。

## 維護方式

- **硬分支**：不追著 upstream 合併。每個月看一次 upstream 的修正，需要的用 `git cherry-pick` 挑進來。
  - `git fetch upstream && git log main..upstream/main --oneline`
- **我們的程式碼盡量放在新檔案**，只在 Handy 原本的流程裡加最少的接點。
  - Handy 最常改的檔案（`actions.rs`、`settings.rs`、`overlay.rs`、`lib.rs`、`shortcut/`）我們改得越少，挑修正時越不會衝突。
- 每個對 Handy 原本檔案的改動，都在註解標 `Yuyin fork:`，方便搜尋。

## 授權

- 保留原本的 `LICENSE`（Copyright (c) 2025 CJ Pais）。App 的「關於」頁面（`src/yuyin/YuyinAbout.tsx`）會顯示 Handy 的致謝和完整授權條款，條款內容直接讀這個 `LICENSE` 檔。
- Handy 的「新功能」彈窗拿掉了（內容是 Handy 的版本紀錄）。版本號從 0.1.0 重新開始。
- MIT 不包含商標，所以不能用「Handy」這個名字。App 名稱（默契／Moqi）、識別碼（`tw.yuyin.dictation`）都已經換掉，自動更新也關了。
  - 自動更新在 `settings.rs` 的 `update_checks_forced_disabled()`。
- 模型下載目前還是走 Handy 的伺服器（`blob.handy.computer`）。之後要改成自己的來源，或使用者自行下載。
