// Yuyin fork: the About page, replacing Handy's AboutSettings. An Apple-style
// header (icon, name, tagline, version), the app-level settings, and the
// credits — including Handy's MIT notice, which the license requires us to
// ship with the app.
import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { Button } from "@/components/ui/Button";
import { AppDataDirectory } from "@/components/settings/AppDataDirectory";
import { AppLanguageSelector } from "@/components/settings/AppLanguageSelector";
import { ThemeSelector } from "@/components/settings/ThemeSelector";
import { LogDirectory } from "@/components/settings/debug";
import { YuyinMark } from "./YuyinLogo";
import handyLicense from "../../LICENSE?raw";

const Credit: React.FC<{
  title: string;
  description: string;
  children?: React.ReactNode;
}> = ({ title, description, children }) => (
  <div className="flex items-center gap-4 min-h-11 px-3.5 py-2.5">
    <div className="flex-1 min-w-0">
      <h3 className="text-[13px] text-text">{title}</h3>
      <p className="mt-0.5 text-[12px] leading-snug text-text/55">
        {description}
      </p>
    </div>
    {children && <div className="flex gap-2 shrink-0">{children}</div>}
  </div>
);

export const YuyinAbout: React.FC = () => {
  const { t } = useTranslation();
  const [version, setVersion] = useState("");
  const [showLicense, setShowLicense] = useState(false);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch((e) => console.error("Failed to get app version:", e));
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="flex flex-col items-center pt-4 pb-2 select-none">
        <YuyinMark
          size={88}
          className="drop-shadow-[0_8px_16px_rgba(0,0,0,0.2)]"
        />
        <h2 className="mt-3 text-[22px] font-bold tracking-[-0.02em] text-text">
          {t("appName")}
        </h2>
        <p className="mt-1 text-[14px] text-text/65">
          {t("settings.about.tagline")}
        </p>
        {version && (
          <p className="mt-2 text-[12px] text-text/40 tabular-nums">
            {t("settings.about.versionLabel", { version })}
          </p>
        )}
      </div>

      <SettingsGroup>
        <AppLanguageSelector descriptionMode="tooltip" grouped={true} />
        <ThemeSelector descriptionMode="tooltip" grouped={true} />
        <AppDataDirectory descriptionMode="tooltip" grouped={true} />
        <LogDirectory grouped={true} />
      </SettingsGroup>

      <SettingsGroup title={t("settings.about.acknowledgments.title")}>
        <Credit
          title={t("settings.about.acknowledgments.handy.title")}
          description={t("settings.about.acknowledgments.handy.description")}
        >
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowLicense((v) => !v)}
          >
            {showLicense
              ? t("settings.about.acknowledgments.handy.hideLicense")
              : t("settings.about.acknowledgments.handy.showLicense")}
          </Button>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => openUrl("https://github.com/cjpais/Handy")}
          >
            {t("settings.about.acknowledgments.handy.source")}
          </Button>
        </Credit>
        {showLicense && (
          <pre className="px-3.5 py-3 text-[11px] leading-relaxed text-text/60 whitespace-pre-wrap font-mono select-text">
            {handyLicense.trim()}
          </pre>
        )}
        <Credit
          title={t("settings.about.acknowledgments.qwen.title")}
          description={t("settings.about.acknowledgments.qwen.description")}
        />
        <Credit
          title={t("settings.about.acknowledgments.ggml.title")}
          description={t("settings.about.acknowledgments.ggml.details")}
        />
      </SettingsGroup>
    </div>
  );
};
