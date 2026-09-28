import React from "react";
import { useTranslation } from "react-i18next";
import {
  Cog,
  FlaskConical,
  History,
  Info,
  Sparkles,
  Cpu,
  SlidersHorizontal,
} from "lucide-react";
// Yuyin fork: our logo instead of Handy's
import YuyinLogo from "@/yuyin/YuyinLogo";
import { YuyinAbout } from "@/yuyin/YuyinAbout";
import { useSettings } from "../hooks/useSettings";
import {
  GeneralSettings,
  AdvancedSettings,
  HistorySettings,
  DebugSettings,
  PostProcessingSettings,
  ModelsSettings,
  YuyinSettings,
} from "./settings";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  [key: string]: any;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  /** Yuyin fork: the colored squircle behind the icon, as in System Settings. */
  color: string;
  component: React.ComponentType;
  enabled: (settings: any) => boolean;
}

export const SECTIONS_CONFIG = {
  general: {
    labelKey: "sidebar.general",
    icon: SlidersHorizontal,
    color: "#8e8e93",
    component: GeneralSettings,
    enabled: () => true,
  },
  // Yuyin fork: context-aware clean-up settings
  yuyin: {
    labelKey: "sidebar.yuyin",
    icon: Sparkles,
    color: "linear-gradient(180deg, #3a3a3f, #0e0e10)",
    component: YuyinSettings,
    enabled: () => true,
  },
  history: {
    labelKey: "sidebar.history",
    icon: History,
    color: "#ff9500",
    component: HistorySettings,
    enabled: () => true,
  },
  models: {
    labelKey: "sidebar.models",
    icon: Cpu,
    color: "#007aff",
    component: ModelsSettings,
    enabled: () => true,
  },
  advanced: {
    labelKey: "sidebar.advanced",
    icon: Cog,
    color: "#636366",
    component: AdvancedSettings,
    enabled: () => true,
  },
  postprocessing: {
    labelKey: "sidebar.postProcessing",
    icon: Sparkles,
    color: "#af52de",
    component: PostProcessingSettings,
    enabled: (settings) => settings?.post_process_enabled ?? false,
  },
  debug: {
    labelKey: "sidebar.debug",
    icon: FlaskConical,
    color: "#ff3b30",
    component: DebugSettings,
    enabled: (settings) => settings?.debug_mode ?? false,
  },
  about: {
    labelKey: "sidebar.about",
    icon: Info,
    color: "#8e8e93",
    // Yuyin fork: our About page (tagline, credits, Handy's license)
    component: YuyinAbout,
    enabled: () => true,
  },
} as const satisfies Record<string, SectionConfig>;

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();
  const { settings } = useSettings();

  const availableSections = Object.entries(SECTIONS_CONFIG)
    .filter(([_, config]) => config.enabled(settings))
    .map(([id, config]) => ({ id: id as SidebarSection, ...config }));

  // Yuyin fork: System Settings–style sidebar. Transparent so macOS's sidebar
  // material shows through; the top 52 px sit under the traffic lights.
  return (
    <div className="flex flex-col w-52 h-full border-e border-hairline shrink-0">
      <div data-tauri-drag-region className="h-[52px] shrink-0" />
      <div className="flex px-4 pb-4">
        <YuyinLogo width={92} />
      </div>
      <nav className="flex flex-col gap-0.5 px-2.5">
        {availableSections.map((section) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;

          return (
            <button
              key={section.id}
              type="button"
              className={`flex gap-2.5 items-center px-2 py-[5px] w-full rounded-md text-start text-[13px] transition-colors ${
                isActive
                  ? "bg-logo-primary text-white"
                  : "hover:bg-black/5 dark:hover:bg-white/10"
              }`}
              onClick={() => onSectionChange(section.id)}
              aria-current={isActive ? "page" : undefined}
            >
              <span
                className="w-[22px] h-[22px] rounded-[6px] grid place-items-center text-white shrink-0 shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.12)]"
                style={{ background: section.color }}
              >
                <Icon width={14} height={14} strokeWidth={2.2} />
              </span>
              <span className="truncate" title={t(section.labelKey)}>
                {t(section.labelKey)}
              </span>
            </button>
          );
        })}
      </nav>
    </div>
  );
};
