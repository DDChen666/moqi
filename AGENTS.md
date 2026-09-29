# AGENTS.md

Guidance for AI coding assistants working in this repository. Human contributors: see [CONTRIBUTING.md](CONTRIBUTING.md).

## What this is

默契 Moqi is a local-first Mandarin/English dictation app for macOS, forked from [Handy](https://github.com/cjpais/Handy). Tauri 2: Rust backend in `src-tauri/`, React + TypeScript frontend in `src/`.

- Our product layer: `src-tauri/src/yuyin/` and `src/yuyin/`. Architecture and the flow of one dictation: [docs/架構.md](docs/架構.md).
- Edits to upstream Handy files are small hooks marked with a `Yuyin fork:` comment explaining why. Keep it that way ([FORK.md](FORK.md)).
- Internal names: the binary is `handy`, the code name is `yuyin`, the bundle id is `tw.yuyin.dictation`. Do not rename them.

## Commands

```bash
bun install
bun run tauri dev                                   # run
cd src-tauri && cargo test --release --lib          # Rust tests
cargo fmt                                           # in src-tauri/
bun x prettier --write . && bun run lint && bun x tsc --noEmit
```

Release-style build (a fixed signing identity keeps macOS permissions across rebuilds; see [BUILD.md](BUILD.md)):

```bash
sh scripts/signing_identity.sh "Moqi Dev"
APPLE_SIGNING_IDENTITY="Moqi Dev" bun run tauri build --bundles app
```

## Rules that matter here

- **Privacy is the product.** Audio never leaves the machine. Anything sent over the network must appear in the history entry's "what was sent" and in [docs/隱私.md](docs/隱私.md). The API key lives only in the Keychain (`yuyin/secrets.rs`); never write it to files or logs, never log transcripts (logs show `[REDACTED]`).
- **Never lose a draft.** Every failure path (network, clean-up, focus change) must still hand the user their text, pasted or copied.
- **The clean-up prompt is pinned by tests** (`src-tauri/src/yuyin/testdata/prompt_v3_*.txt`). Change the prompt only with a measured reason and update the fixtures in the same change.
- **UI strings go through i18next.** Add keys to `src/i18n/locales/zh-TW/translation.json` and `en/translation.json`; ESLint rejects hard-coded JSX strings. zh-TW copy uses Taiwan wording.
- Handle errors explicitly in Rust (no `unwrap()` on production paths). New behaviour needs a unit test.
- Commits: conventional prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`); the body says why.

## Pull requests and issues

Follow [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md) and the issue forms in [.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/). State in the PR how the change was verified and whether it sends any new data.

## CLI flags (inherited from Handy)

| Flag                     | Description                            |
| ------------------------ | -------------------------------------- |
| `--toggle-transcription` | Toggle recording on a running instance |
| `--cancel`               | Cancel the current operation           |
| `--start-hidden`         | Launch without showing the window      |
| `--no-tray`              | Launch without the menu bar icon       |
| `--debug`                | Verbose logging                        |

Debug features in the app window: `⌘⇧D`.
