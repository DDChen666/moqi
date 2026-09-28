// Yuyin fork: typed wrappers for the commands in
// src-tauri/src/yuyin/commands.rs. Kept separate from the generated
// bindings.ts so our frontend code never depends on regenerating it.
import { invoke } from "@tauri-apps/api/core";

/** 原話 / 整理 / 潤飾 */
export type Level = "raw" | "tidy" | "polish";

export interface YuyinConfig {
  level: Level;
  vocab: string[];
  base_url: string;
  model: string;
  timeout_ms: number;
}

export const yuyinApi = {
  getConfig: () => invoke<YuyinConfig>("yuyin_get_config"),
  setConfig: (config: YuyinConfig) =>
    invoke<void>("yuyin_set_config", { config }),
  /** The key itself never reaches the frontend; only whether one is stored. */
  hasApiKey: () => invoke<boolean>("yuyin_has_api_key"),
  /** An empty string removes the key from the Keychain. */
  setApiKey: (key: string) => invoke<void>("yuyin_set_api_key", { key }),
  testPolish: (text: string) => invoke<string>("yuyin_test_polish", { text }),
};
