// Yuyin fork: every setting Moqi has — shortcut, clean-up level, clean-up
// service and key, microphone, general — plus About. Handy's other settings
// keep the defaults written by yuyin/defaults.rs.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { ShortcutInput } from "@/components/settings/ShortcutInput";
import { MicrophoneSelector } from "@/components/settings/MicrophoneSelector";
import { useSettings } from "@/hooks/useSettings";
import { useOsType } from "@/hooks/useOsType";
import { getSupportedLanguage } from "@/i18n";
import { applyTheme, THEME_OPTIONS } from "@/lib/utils/theme";
import { yuyinApi, type Level, type Service, type YuyinConfig } from "../api";
import { YuyinAbout } from "../YuyinAbout";
import {
  Group,
  Kbd,
  Row,
  Segmented,
  Select,
  SmallButton,
  Switch,
  TextInput,
} from "./ui";
import { BackIcon, ChevronIcon } from "./icons";
import { tapKeyLabel } from "./format";

const DEEPSEEK = {
  base_url: "https://api.deepseek.com",
  model: "deepseek-flash",
};

const ApiKeyRow: React.FC = () => {
  const { t } = useTranslation();
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [editing, setEditing] = useState(false);
  const [key, setKey] = useState("");
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<{ ok: boolean; text: string } | null>(
    null,
  );

  useEffect(() => {
    yuyinApi
      .hasApiKey()
      .then(setHasKey)
      .catch(() => setHasKey(false));
  }, []);

  const save = async () => {
    try {
      await yuyinApi.setApiKey(key.trim());
      setHasKey(key.trim().length > 0);
      setKey("");
      setEditing(false);
      toast.success(t("moqi.settings.keySaved"));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const test = async () => {
    setTesting(true);
    setResult(null);
    try {
      const text = await yuyinApi.testPolish(t("yuyin.test.sample"));
      setResult({ ok: true, text });
    } catch (e) {
      setResult({ ok: false, text: String(e) });
    } finally {
      setTesting(false);
    }
  };

  return (
    <>
      <Row label={t("moqi.settings.apiKey")}>
        {editing ? (
          <>
            <TextInput
              type="password"
              value={key}
              autoFocus
              onChange={(e) => setKey(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && save()}
              placeholder="sk-…"
              aria-label={t("moqi.settings.apiKey")}
              className="w-[220px] !py-1 !text-[12px]"
            />
            <SmallButton onClick={save} disabled={!key.trim()}>
              {t("moqi.settings.save")}
            </SmallButton>
            <SmallButton onClick={() => setEditing(false)}>
              {t("moqi.settings.cancel")}
            </SmallButton>
          </>
        ) : (
          <>
            <span
              className={`text-[12px] ${hasKey ? "text-positive" : "text-muted"}`}
            >
              {hasKey === null
                ? ""
                : hasKey
                  ? t("moqi.settings.keyStored")
                  : t("moqi.settings.keyMissing")}
            </span>
            <SmallButton onClick={() => setEditing(true)}>
              {hasKey
                ? t("moqi.settings.replace")
                : t("moqi.settings.enterKey")}
            </SmallButton>
            <SmallButton onClick={test} disabled={!hasKey || testing}>
              {testing ? t("moqi.settings.testing") : t("moqi.settings.test")}
            </SmallButton>
          </>
        )}
      </Row>
      {result && (
        <div
          className={`px-3.5 py-2.5 text-[12px] leading-relaxed select-text ${
            result.ok ? "text-text" : "text-error"
          }`}
        >
          {result.ok
            ? t("moqi.settings.testResult", { text: result.text })
            : result.text}
        </div>
      )}
    </>
  );
};

const CustomServiceRow: React.FC<{
  config: YuyinConfig;
  save: (c: YuyinConfig) => void;
}> = ({ config, save }) => {
  const { t } = useTranslation();
  const [baseUrl, setBaseUrl] = useState(config.base_url);
  const [model, setModel] = useState(config.model);
  const commit = () => {
    if (baseUrl !== config.base_url || model !== config.model) {
      save({ ...config, base_url: baseUrl.trim(), model: model.trim() });
    }
  };
  return (
    <Row label={t("moqi.settings.endpoint")} htmlFor="moqi-base-url">
      <TextInput
        id="moqi-base-url"
        value={baseUrl}
        onChange={(e) => setBaseUrl(e.target.value)}
        onBlur={commit}
        placeholder="https://…/v1"
        className="w-[190px] !py-1 !text-[12px]"
      />
      <TextInput
        value={model}
        onChange={(e) => setModel(e.target.value)}
        onBlur={commit}
        placeholder={t("moqi.settings.modelName")}
        aria-label={t("moqi.settings.modelName")}
        className="w-[120px] !py-1 !text-[12px]"
      />
    </Row>
  );
};

export const SettingsPage: React.FC = () => {
  const { t, i18n } = useTranslation();
  const os = useOsType();
  const { settings, updateSetting } = useSettings();
  const [config, setConfig] = useState<YuyinConfig | null>(null);
  const [about, setAbout] = useState(false);

  useEffect(() => {
    yuyinApi
      .getConfig()
      .then(setConfig)
      .catch((e) => console.error(e));
  }, []);

  const save = async (next: YuyinConfig) => {
    setConfig(next);
    try {
      await yuyinApi.setConfig(next);
    } catch (e) {
      toast.error(String(e));
    }
  };

  if (about) {
    return (
      <div className="flex flex-col gap-4">
        <button
          type="button"
          onClick={() => setAbout(false)}
          className="self-start inline-flex items-center gap-1 text-[13px] text-logo-primary"
        >
          <BackIcon size={13} />
          {t("moqi.nav.settings")}
        </button>
        <YuyinAbout />
      </div>
    );
  }

  const service: Service = config?.service ?? "deepseek";
  const localOnly = service === "none";
  const level: Level = localOnly ? "raw" : (config?.level ?? "tidy");
  const language =
    getSupportedLanguage(settings?.app_language) || i18n.language;
  const tapKey = tapKeyLabel(
    settings?.bindings?.transcribe?.current_binding,
    os,
    t,
  );

  return (
    <div className="flex flex-col gap-[18px] max-w-[600px]">
      <h1 className="m-0 mb-1 text-[26px] font-bold tracking-[-0.02em] text-text">
        {t("moqi.nav.settings")}
      </h1>

      <Group title={t("moqi.settings.shortcuts")}>
        <ShortcutInput
          shortcutId="transcribe"
          grouped
          descriptionMode="inline"
        />
        <Row
          label={t("moqi.settings.handsFree")}
          description={t("moqi.settings.handsFreeHint")}
        >
          <span className="flex gap-[3px]">
            <Kbd>{tapKey}</Kbd>
            <Kbd>{tapKey}</Kbd>
          </span>
        </Row>
        <Row label={t("moqi.settings.cancelRecording")}>
          <Kbd>{t("moqi.keys.esc")}</Kbd>
        </Row>
      </Group>

      <Group title={t("moqi.settings.level")}>
        <div className="px-3.5 py-3 flex flex-col gap-2.5">
          <Segmented
            label={t("moqi.settings.level")}
            value={level}
            disabled={!config || localOnly}
            onChange={(v) => config && save({ ...config, level: v })}
            options={(["raw", "tidy", "polish"] as Level[]).map((v) => ({
              value: v,
              label: t(`moqi.levels.${v}`),
            }))}
          />
          <p className="m-0 text-[12px] leading-relaxed text-text/80">
            {t(`moqi.settings.levelNote.${level}`)}
          </p>
        </div>
      </Group>

      <Group
        title={t("moqi.settings.service")}
        footnote={
          localOnly
            ? t("moqi.settings.serviceNoteLocal")
            : t("moqi.settings.serviceNote")
        }
      >
        <Row label={t("moqi.settings.serviceLabel")} htmlFor="moqi-service">
          <Select
            id="moqi-service"
            value={service}
            disabled={!config}
            onChange={(e) => {
              if (!config) return;
              const next = e.target.value as Service;
              save(
                next === "deepseek"
                  ? { ...config, service: next, ...DEEPSEEK }
                  : { ...config, service: next },
              );
            }}
          >
            <option value="deepseek">
              {t("moqi.settings.serviceDeepseek")}
            </option>
            <option value="custom">{t("moqi.settings.serviceCustom")}</option>
            <option value="none">{t("moqi.settings.serviceNone")}</option>
          </Select>
        </Row>
        {!localOnly && <ApiKeyRow />}
        {config && service === "custom" && (
          <CustomServiceRow config={config} save={save} />
        )}
      </Group>

      <Group title={t("moqi.settings.microphone")}>
        <MicrophoneSelector grouped descriptionMode="inline" />
        <Row
          label={t("moqi.settings.sound")}
          description={t("moqi.settings.soundHint")}
        >
          <Switch
            label={t("moqi.settings.sound")}
            checked={settings?.audio_feedback ?? true}
            onChange={(v) => updateSetting("audio_feedback", v)}
          />
        </Row>
      </Group>

      <Group title={t("moqi.settings.general")}>
        <Row label={t("moqi.settings.autostart")}>
          <Switch
            label={t("moqi.settings.autostart")}
            checked={settings?.autostart_enabled ?? false}
            onChange={(v) => updateSetting("autostart_enabled", v)}
          />
        </Row>
        <Row label={t("moqi.settings.language")} htmlFor="moqi-language">
          <Select
            id="moqi-language"
            value={language}
            onChange={(e) => {
              i18n.changeLanguage(e.target.value);
              updateSetting("app_language", e.target.value);
            }}
          >
            {/* Language names are shown in their own language. */}
            {/* eslint-disable-next-line i18next/no-literal-string */}
            <option value="zh-TW">繁體中文</option>
            {/* eslint-disable-next-line i18next/no-literal-string */}
            <option value="en">English</option>
          </Select>
        </Row>
        <Row label={t("moqi.settings.appearance")} htmlFor="moqi-theme">
          <Select
            id="moqi-theme"
            value={settings?.theme ?? "system"}
            onChange={(e) => {
              const theme = e.target.value as (typeof THEME_OPTIONS)[number];
              applyTheme(theme);
              updateSetting("theme", theme);
            }}
          >
            {THEME_OPTIONS.map((v) => (
              <option key={v} value={v}>
                {t(`theme.options.${v}`)}
              </option>
            ))}
          </Select>
        </Row>
        <button
          type="button"
          onClick={() => setAbout(true)}
          className="w-full flex items-center justify-between min-h-11 px-3.5 text-[13px] text-text text-start"
        >
          {t("moqi.settings.about")}
          <ChevronIcon size={12} className="text-muted" />
        </button>
      </Group>
    </div>
  );
};
