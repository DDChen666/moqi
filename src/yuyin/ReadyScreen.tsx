// Yuyin fork: setup step 3. The slogan, and a box to try dictation right
// here: the focused box is Moqi's own, so the first result lands in front of
// the user instead of in some other app.
import React, { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useOsType } from "@/hooks/useOsType";
import { useSettings } from "@/hooks/useSettings";
import { YuyinMark } from "./YuyinLogo";
import { StepDots } from "./onboarding/PolishStep";
import { PrimaryButton } from "./window/ui";
import { keyLabel } from "./window/format";
import "./springs";
import "./onboarding.css";

export const ReadyScreen: React.FC<{ onStart: () => void }> = ({ onStart }) => {
  const { t } = useTranslation();
  const os = useOsType();
  const { settings } = useSettings();
  const [text, setText] = useState("");
  const boxRef = useRef<HTMLTextAreaElement>(null);
  const talkKey = keyLabel(
    settings?.bindings?.transcribe?.current_binding,
    os,
    t,
  );

  useEffect(() => {
    const id = setTimeout(() => boxRef.current?.focus(), 900);
    return () => clearTimeout(id);
  }, []);

  return (
    <div className="yy-ready h-screen w-full flex flex-col items-center justify-center p-8 bg-background">
      <div data-tauri-drag-region className="fixed top-0 inset-x-0 h-[52px]" />
      <StepDots step={3} />
      <div
        className="yy-ready-icon"
        style={{ "--i": 0 } as React.CSSProperties}
      >
        <YuyinMark size={76} />
      </div>
      <h1
        className="yy-rise mt-5 mb-0 text-[26px] font-bold tracking-[-0.02em] text-text"
        style={{ "--i": 1 } as React.CSSProperties}
      >
        {t("onboarding.ready.title")}
      </h1>
      <p
        className="yy-rise mt-2 mb-0 text-[15px] text-muted text-center"
        style={{ "--i": 2 } as React.CSSProperties}
      >
        {t("onboarding.ready.slogan")}
      </p>

      <div
        className="yy-rise mt-7 w-full max-w-[520px] flex flex-col gap-2"
        style={{ "--i": 3 } as React.CSSProperties}
      >
        <div className="flex justify-between items-baseline px-0.5">
          <label
            htmlFor="moqi-try"
            className="text-[12px] font-semibold text-text"
          >
            {t("moqi.onboarding.tryTitle")}
          </label>
          <span className="text-[11px] text-muted">
            {text
              ? t("moqi.onboarding.tryDone")
              : t("moqi.onboarding.tryHint", { key: talkKey })}
          </span>
        </div>
        <textarea
          id="moqi-try"
          ref={boxRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          rows={3}
          placeholder={t("moqi.onboarding.tryPlaceholder", { key: talkKey })}
          className="w-full resize-none px-4 py-3.5 rounded-[12px] bg-surface text-[15px] leading-relaxed text-text placeholder:text-muted outline-none shadow-[0_0_0_0.5px_var(--color-hairline)] focus:shadow-[0_0_0_2px_var(--color-logo-primary),0_4px_14px_rgba(0,113,227,0.1)] select-text"
        />
        <p className="m-0 text-center text-[12px] text-muted">
          {t("moqi.onboarding.tryIdeas")}
        </p>
      </div>

      <PrimaryButton
        className="yy-rise mt-7"
        style={{ "--i": 4 } as React.CSSProperties}
        onClick={onStart}
      >
        {t("onboarding.ready.start")}
      </PrimaryButton>
    </div>
  );
};
