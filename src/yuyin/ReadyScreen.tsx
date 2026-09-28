// Yuyin fork: the screen after the last permission is granted. It carries
// the slogan and shows the one thing to remember — the key — before the
// settings window takes over.
import React, { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { YuyinMark } from "./YuyinLogo";
import "./springs";
import "./onboarding.css";

export const ReadyScreen: React.FC<{ onStart: () => void }> = ({ onStart }) => {
  const { t } = useTranslation();

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Enter") onStart();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onStart]);

  return (
    <div className="yy-ready h-screen w-full flex flex-col items-center justify-center p-8 bg-background">
      <div data-tauri-drag-region className="fixed top-0 inset-x-0 h-[52px]" />
      <div
        className="yy-ready-icon"
        style={{ "--i": 0 } as React.CSSProperties}
      >
        <YuyinMark size={76} />
      </div>
      <h1
        className="yy-rise mt-5 text-[26px] font-bold tracking-[-0.02em] text-text"
        style={{ "--i": 1 } as React.CSSProperties}
      >
        {t("onboarding.ready.title")}
      </h1>
      <p
        className="yy-rise mt-2 text-[15px] text-text/65 text-center"
        style={{ "--i": 2 } as React.CSSProperties}
      >
        {t("onboarding.ready.slogan")}
      </p>
      <div
        className="yy-rise mt-9"
        style={{ "--i": 3 } as React.CSSProperties}
        aria-hidden="true"
      >
        <kbd className="yy-key">
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="yy-key-glyph">⌥</span>
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span className="yy-key-name">option</span>
        </kbd>
      </div>
      <button
        type="button"
        onClick={onStart}
        className="yy-rise mt-10 min-w-[140px] px-6 py-[7px] rounded-full bg-logo-primary text-white text-[14px] font-medium hover:brightness-110 active:brightness-95 transition focus:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary/40 focus-visible:ring-offset-2"
        style={{ "--i": 4 } as React.CSSProperties}
      >
        {t("onboarding.ready.start")}
      </button>
    </div>
  );
};
