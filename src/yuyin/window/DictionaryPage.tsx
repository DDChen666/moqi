// Yuyin fork: the personal dictionary — names and terms the clean-up must
// spell the user's way. Stored in yuyin.json (config.vocab).
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { yuyinApi, type YuyinConfig } from "../api";
import { Card, PageTitle, TextInput } from "./ui";

export const DictionaryPage: React.FC = () => {
  const { t } = useTranslation();
  const [config, setConfig] = useState<YuyinConfig | null>(null);
  const [draft, setDraft] = useState("");

  useEffect(() => {
    yuyinApi
      .getConfig()
      .then(setConfig)
      .catch((e) => console.error("Failed to load dictionary:", e));
  }, []);

  const save = async (vocab: string[]) => {
    if (!config) return;
    const next = { ...config, vocab };
    setConfig(next);
    try {
      await yuyinApi.setConfig(next);
    } catch (e) {
      console.error("Failed to save dictionary:", e);
      toast.error(t("moqi.dictionary.saveFailed"));
    }
  };

  const add = () => {
    const word = draft.trim();
    if (!config || !word) return;
    if (!config.vocab.includes(word)) save([...config.vocab, word]);
    setDraft("");
  };

  return (
    <div className="flex flex-col gap-5">
      <PageTitle
        title={t("moqi.nav.dictionary")}
        subtitle={t("moqi.dictionary.subtitle")}
      />

      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          add();
        }}
      >
        <TextInput
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder={t("moqi.dictionary.placeholder")}
          aria-label={t("moqi.dictionary.placeholder")}
          className="flex-1"
        />
        <button
          type="submit"
          disabled={!draft.trim()}
          className="text-[13px] font-medium px-[18px] rounded-[9px] bg-logo-primary text-white disabled:opacity-40 hover:brightness-110 transition"
        >
          {t("moqi.dictionary.add")}
        </button>
      </form>

      <Card className="p-4 flex flex-col gap-3">
        <div className="flex justify-between items-baseline">
          <span className="text-[12px] font-semibold text-text">
            {t("moqi.dictionary.count", { count: config?.vocab.length ?? 0 })}
          </span>
          <span className="text-[11px] text-muted">
            {t("moqi.dictionary.removeHint")}
          </span>
        </div>
        <div className="flex flex-wrap gap-1.5">
          {config?.vocab.map((word) => (
            <span
              key={word}
              className="inline-flex items-center gap-0.5 text-[12.5px] ps-2.5 pe-1 py-1 rounded-[7px] bg-fill text-text select-text"
            >
              {word}
              <button
                type="button"
                onClick={() => save(config.vocab.filter((w) => w !== word))}
                aria-label={t("moqi.dictionary.remove", { word })}
                className="w-[18px] h-[18px] rounded-[5px] text-muted hover:text-text hover:bg-black/5 dark:hover:bg-white/10 leading-none"
              >
                ×
              </button>
            </span>
          ))}
        </div>
      </Card>
    </div>
  );
};
