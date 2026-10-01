// Yuyin fork: typed wrappers for the commands in
// src-tauri/src/yuyin/commands.rs. Kept separate from the generated
// bindings.ts so our frontend code never depends on regenerating it.
import { invoke } from "@tauri-apps/api/core";

/** 原話 / 整理 / 潤飾 */
export type Level = "raw" | "tidy" | "polish";

/** Where the clean-up runs: DeepSeek, a model on OpenRouter, any
 * OpenAI-compatible service, or nowhere. */
export type Service = "deepseek" | "openrouter" | "custom" | "none";

/** One model in OpenRouter's catalog; prices in US$ per million tokens. */
export interface ModelInfo {
  id: string;
  name: string;
  input_price: number;
  output_price: number;
}

export interface YuyinConfig {
  level: Level;
  service: Service;
  vocab: string[];
  base_url: string;
  model: string;
  timeout_ms: number;
}

/** Home page numbers, computed locally from the timings log (stats.rs). */
export interface YuyinStats {
  dictations: number;
  chars: number;
  speaking_ms: number;
  saved_ms: number;
  chars_per_minute: number;
  active_days: number;
  current_streak: number;
  longest_streak: number;
  /** The last 26 weeks, oldest first, starting on a Sunday. */
  days: { date: string; dictations: number }[];
  privacy: {
    audio_uploaded_ms: number;
    app_names_sent: number;
    text_sent_chars: number;
    sent_to: string[];
  };
}

/** What the history page shows under an entry. */
export interface EntryMeta {
  app: string;
  context: "chat" | "to_ai" | "notes" | "other";
  level: Level | null;
  polish: "skipped" | "ok" | "failed";
  sent_chars: number;
  sent_to: string | null;
  spoke_ms: number | null;
  output_ms: number | null;
}

export const yuyinApi = {
  getConfig: () => invoke<YuyinConfig>("yuyin_get_config"),
  setConfig: (config: YuyinConfig) =>
    invoke<void>("yuyin_set_config", { config }),
  /** The key itself never reaches the frontend; only whether one is stored
   * for the service at `baseUrl` (default: the saved service). Each service
   * has its own key. */
  hasApiKey: (baseUrl?: string) =>
    invoke<boolean>("yuyin_has_api_key", { baseUrl }),
  /** Saved for the service at `baseUrl`; an empty string removes it. */
  setApiKey: (key: string, baseUrl?: string) =>
    invoke<void>("yuyin_set_api_key", { key, baseUrl }),
  testPolish: (text: string) => invoke<string>("yuyin_test_polish", { text }),
  /** OpenRouter's public catalog (no key needed). */
  openrouterModels: () => invoke<ModelInfo[]>("yuyin_openrouter_models"),
  stats: () => invoke<YuyinStats>("yuyin_stats"),
  /** Keyed by the history entry's recording file name. */
  historyMeta: () => invoke<Record<string, EntryMeta>>("yuyin_history_meta"),
  /** History's 重貼: back to the previous app, then paste. */
  repaste: (text: string) => invoke<void>("yuyin_repaste", { text }),
  reportError: (message: string) =>
    invoke<void>("yuyin_report_error", { message }).catch(() => {}),
};
