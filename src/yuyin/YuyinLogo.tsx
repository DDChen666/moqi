// Yuyin fork: our mark (the obsidian icon: a voice wave that calms into a
// line and stops at a caret) plus the wordmark. Replaces HandyTextLogo.
// Source artwork: yuyin/brand/render.html.
import React, { useId } from "react";

/** Working name; the product name isn't translated. */
export const PRODUCT_NAME = "Yuyin";

const WAVE =
  "M250 512 C 282 382, 338 382, 370 512 C 400 636, 458 636, 488 512 C 512 432, 560 432, 584 512 C 602 566, 630 566, 646 512 L 688 512";

export const YuyinMark: React.FC<{ size?: number; className?: string }> = ({
  size = 28,
  className,
}) => {
  const id = useId().replace(/:/g, "");
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 1024 1024"
      className={className}
      aria-hidden="true"
    >
      <defs>
        <linearGradient id={`${id}bg`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#3a3a3f" />
          <stop offset="0.42" stopColor="#18181b" />
          <stop offset="1" stopColor="#060607" />
        </linearGradient>
        <linearGradient
          id={`${id}st`}
          gradientUnits="userSpaceOnUse"
          x1="250"
          y1="0"
          x2="760"
          y2="0"
        >
          <stop offset="0" stopColor="#9c9ca3" />
          <stop offset="0.7" stopColor="#e8e8ed" />
          <stop offset="1" stopColor="#ffffff" />
        </linearGradient>
      </defs>
      <rect
        x="100"
        y="100"
        width="824"
        height="824"
        rx="185"
        fill={`url(#${id}bg)`}
      />
      <path
        d={WAVE}
        fill="none"
        stroke={`url(#${id}st)`}
        strokeWidth="44"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <rect x="730" y="328" width="36" height="368" rx="18" fill="#fff" />
      <rect
        x="101.5"
        y="101.5"
        width="821"
        height="821"
        rx="184"
        fill="none"
        stroke="#fff"
        strokeOpacity="0.14"
        strokeWidth="4"
      />
    </svg>
  );
};

/** Mark + wordmark. `width` matches HandyTextLogo's prop for drop-in use. */
const YuyinLogo: React.FC<{ width?: number; className?: string }> = ({
  width = 120,
  className = "",
}) => {
  const mark = Math.round(width * 0.26);
  return (
    <div
      className={`flex items-center justify-center ${className}`}
      style={{ gap: Math.round(width * 0.07) }}
    >
      <YuyinMark size={mark} />
      <span
        className="font-semibold text-text"
        style={{ fontSize: Math.round(width * 0.17), letterSpacing: "-0.02em" }}
      >
        {PRODUCT_NAME}
      </span>
    </div>
  );
};

export default YuyinLogo;
