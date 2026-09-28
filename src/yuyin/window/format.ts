// Yuyin fork: number, duration, key and date labels for the 1.0 window.
import type { TFunction } from "i18next";
import { formatKeyCombination, type OSType } from "@/lib/utils/keyboard";

/** 12480 → "12,480" */
export const formatCount = (n: number, locale: string): string =>
  new Intl.NumberFormat(locale).format(Math.round(n));

/**
 * A duration as big-number/unit pairs, for "3 小時 58 分" with small units:
 * under a minute in seconds, under an hour in minutes, else hours+minutes.
 */
export const durationParts = (
  ms: number,
  t: TFunction,
): { value: string; unit: string }[] => {
  const s = Math.round(ms / 1000);
  if (s < 60) return [{ value: String(s), unit: t("moqi.unit.seconds") }];
  const m = Math.round(s / 60);
  if (m < 60) return [{ value: String(m), unit: t("moqi.unit.minutes") }];
  return [
    { value: String(Math.floor(m / 60)), unit: t("moqi.unit.hours") },
    { value: String(m % 60), unit: t("moqi.unit.minutes") },
  ];
};

/** 12400 ms → "12 秒"; used in history ("說了 12 秒"). */
export const secondsLabel = (ms: number, t: TFunction): string => {
  const s = ms / 1000;
  return t("moqi.unit.secondsValue", {
    value: s < 10 ? s.toFixed(1).replace(/\.0$/, "") : Math.round(s),
  });
};

/**
 * The talk key as people see it on the keyboard: "右 ⌥ Option" on a Mac,
 * "右 Alt" on Windows; anything else through Handy's formatter.
 */
export const keyLabel = (
  binding: string | undefined,
  os: OSType,
  t: TFunction,
): string => {
  if (!binding) return "";
  if (binding === "option_right" || binding === "alt_right") {
    return os === "macos"
      ? t("moqi.keys.rightOption")
      : t("moqi.keys.rightAlt");
  }
  return formatKeyCombination(binding, os);
};

const startOfDay = (d: Date) =>
  new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

/** History groups: 今天 / 昨天 / 9月26日. `ts` is Unix seconds. */
export const dayLabel = (ts: number, locale: string, t: TFunction): string => {
  const day = startOfDay(new Date(ts * 1000));
  const today = startOfDay(new Date());
  if (day === today) return t("moqi.history.today");
  if (day === today - 86_400_000) return t("moqi.history.yesterday");
  return new Intl.DateTimeFormat(locale, {
    month: "long",
    day: "numeric",
  }).format(new Date(ts * 1000));
};

/** "21:42" */
export const timeLabel = (ts: number, locale: string): string =>
  new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(ts * 1000));
