// Yuyin fork: settings for our context-aware clean-up (level, API key,
// personal dictionary) and a button to try it on sample text.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { Dropdown } from "../../ui/Dropdown";
import { Input } from "../../ui/Input";
import { Textarea } from "../../ui/Textarea";
import { Button } from "../../ui/Button";
import { yuyinApi, type Level, type YuyinConfig } from "@/yuyin/api";

const LEVELS: Level[] = ["raw", "tidy", "polish"];

export const YuyinSettings: React.FC = () => {
  const { t } = useTranslation();
  const [config, setConfig] = useState<YuyinConfig | null>(null);
  const [hasKey, setHasKey] = useState(false);
  const [keyInput, setKeyInput] = useState("");
  const [vocabText, setVocabText] = useState("");
  const [sample, setSample] = useState(t("yuyin.test.sample"));
  const [testResult, setTestResult] = useState<{
    ok: boolean;
    text: string;
  } | null>(null);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    yuyinApi
      .getConfig()
      .then((cfg) => {
        setConfig(cfg);
        setVocabText(cfg.vocab.join("\n"));
      })
      .catch((e) => setError(String(e)));
    yuyinApi
      .hasApiKey()
      .then(setHasKey)
      .catch(() => setHasKey(false));
  }, []);

  const save = async (next: YuyinConfig) => {
    setConfig(next);
    try {
      await yuyinApi.setConfig(next);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const saveKey = async (key: string) => {
    try {
      await yuyinApi.setApiKey(key);
      setHasKey(key.trim().length > 0);
      setKeyInput("");
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const saveVocab = () => {
    if (!config) return;
    const vocab = vocabText
      .split("\n")
      .map((w) => w.trim())
      .filter((w) => w.length > 0);
    save({ ...config, vocab });
  };

  const runTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      setTestResult({ ok: true, text: await yuyinApi.testPolish(sample) });
    } catch (e) {
      setTestResult({ ok: false, text: String(e) });
    } finally {
      setTesting(false);
    }
  };

  if (!config) {
    return error ? <p className="text-sm text-red-500">{error}</p> : null;
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("yuyin.title")}>
        <SettingContainer
          title={t("yuyin.level.title")}
          description={t("yuyin.level.description")}
          descriptionMode="tooltip"
          grouped={true}
        >
          <Dropdown
            options={LEVELS.map((level) => ({
              value: level,
              label: t(`yuyin.level.options.${level}`),
              description: t(`yuyin.level.descriptions.${level}`),
            }))}
            selectedValue={config.level}
            onSelect={(value) => save({ ...config, level: value as Level })}
          />
        </SettingContainer>

        <SettingContainer
          title={t("yuyin.apiKey.title")}
          description={t("yuyin.apiKey.description")}
          descriptionMode="inline"
          grouped={true}
          layout="stacked"
        >
          <div className="flex items-center gap-2">
            <Input
              type="password"
              autoComplete="off"
              value={keyInput}
              placeholder={
                hasKey
                  ? t("yuyin.apiKey.placeholderSet")
                  : t("yuyin.apiKey.placeholder")
              }
              onChange={(e) => setKeyInput(e.target.value)}
              className="flex-1"
            />
            <Button
              size="sm"
              disabled={keyInput.trim().length === 0}
              onClick={() => saveKey(keyInput)}
            >
              {t("yuyin.apiKey.save")}
            </Button>
            {hasKey && (
              <Button
                size="sm"
                variant="danger-ghost"
                onClick={() => saveKey("")}
              >
                {t("yuyin.apiKey.clear")}
              </Button>
            )}
          </div>
          <p className="text-xs mt-1 opacity-70">
            {hasKey
              ? t("yuyin.apiKey.statusSet")
              : t("yuyin.apiKey.statusNotSet")}
          </p>
        </SettingContainer>

        <SettingContainer
          title={t("yuyin.vocab.title")}
          description={t("yuyin.vocab.description")}
          descriptionMode="inline"
          grouped={true}
          layout="stacked"
        >
          <Textarea
            value={vocabText}
            rows={6}
            placeholder={t("yuyin.vocab.placeholder")}
            onChange={(e) => setVocabText(e.target.value)}
            onBlur={saveVocab}
            className="w-full"
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("yuyin.test.title")}>
        <SettingContainer
          title={t("yuyin.test.title")}
          description={t("yuyin.test.description")}
          descriptionMode="inline"
          grouped={true}
          layout="stacked"
        >
          <Textarea
            value={sample}
            rows={3}
            onChange={(e) => setSample(e.target.value)}
            className="w-full"
          />
          <div className="flex items-center gap-2 mt-2">
            <Button
              size="sm"
              disabled={testing || !hasKey || config.level === "raw"}
              onClick={runTest}
            >
              {testing ? t("yuyin.test.running") : t("yuyin.test.button")}
            </Button>
            {!hasKey && (
              <span className="text-xs opacity-70">
                {t("yuyin.test.needKey")}
              </span>
            )}
          </div>
          {testResult && (
            <p
              className={`text-sm mt-2 whitespace-pre-wrap ${
                testResult.ok ? "" : "text-red-500"
              }`}
            >
              {testResult.text}
            </p>
          )}
        </SettingContainer>
      </SettingsGroup>

      <p className="text-xs opacity-60 px-1">{t("yuyin.privacy")}</p>
      {error && <p className="text-sm text-red-500 px-1">{error}</p>}
    </div>
  );
};
