// Yuyin fork: the window's line icons, drawn on a 16 px grid to match the
// capsule's context icons.
import React from "react";

type IconProps = { size?: number; className?: string };

const Line: React.FC<IconProps & { children: React.ReactNode }> = ({
  size = 16,
  className,
  children,
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 16 16"
    fill="none"
    stroke="currentColor"
    strokeWidth={1.5}
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
    aria-hidden="true"
  >
    {children}
  </svg>
);

export const HomeIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M2.8 7.2 8 3l5.2 4.2V13a.6.6 0 0 1-.6.6H9.8V10H6.2v3.6H3.4a.6.6 0 0 1-.6-.6Z" />
  </Line>
);

export const ClockIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <circle cx="8" cy="8" r="5.6" />
    <path d="M8 5v3.3l2.1 1.3" />
  </Line>
);

export const BookIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M3.4 3h6.8a1.8 1.8 0 0 1 1.8 1.8V13H5.2a1.8 1.8 0 0 1-1.8-1.8Z" />
    <path d="M3.4 11.2a1.8 1.8 0 0 1 1.8-1.8H12" />
  </Line>
);

export const GearIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <circle cx="8" cy="8" r="2.3" />
    <path d="M8 1.8v1.7M8 12.5v1.7M1.8 8h1.7M12.5 8h1.7M3.6 3.6l1.2 1.2M11.2 11.2l1.2 1.2M3.6 12.4l1.2-1.2M11.2 4.8l1.2-1.2" />
  </Line>
);

export const FlaskIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M6.2 2.2h3.6M6.8 2.2v4L3.4 12.4a.9.9 0 0 0 .8 1.4h7.6a.9.9 0 0 0 .8-1.4L9.2 6.2v-4" />
  </Line>
);

export const LockIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <rect x="3.2" y="7" width="9.6" height="6.8" rx="1.6" />
    <path d="M5.4 7V5.2a2.6 2.6 0 0 1 5.2 0V7" />
  </Line>
);

export const SearchIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <circle cx="7" cy="7" r="4.5" />
    <path d="m10.4 10.4 3.2 3.2" />
  </Line>
);

export const ChevronIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="m6 3 5 5-5 5" />
  </Line>
);

export const BackIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M10 3 5 8l5 5" />
  </Line>
);

export const CheckIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="m3.5 8.4 3 3 6-6.6" />
  </Line>
);

export const MicIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <rect x="5.5" y="1.8" width="5" height="8" rx="2.5" />
    <path d="M3.3 7.6a4.7 4.7 0 0 0 9.4 0M8 12.3v2" />
  </Line>
);

export const DownloadIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M8 2.5v7.5M4.8 7l3.2 3.2L11.2 7M3 13.5h10" />
  </Line>
);

export const AccessIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <circle cx="8" cy="3.2" r="1.3" />
    <path d="M3.5 6h9M8 6v4.2M8 10.2l-2.2 3.8M8 10.2l2.2 3.8" />
  </Line>
);

/** Filled sparkle: "to AI", and the 整理 option. */
export const SparkleIcon: React.FC<IconProps> = ({ size = 16, className }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 16 16"
    fill="currentColor"
    className={className}
    aria-hidden="true"
  >
    <path d="M8 1.8l1.3 3.6 3.6 1.3-3.6 1.3L8 11.6 6.7 8 3.1 6.7l3.6-1.3L8 1.8Zm4.4 8.4.5 1.4 1.4.5-1.4.5-.5 1.4-.5-1.4-1.4-.5 1.4-.5.5-1.4Z" />
  </svg>
);

export const ChatIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M8 2.5c3.4 0 6 2.1 6 4.8S11.4 12 8 12c-.6 0-1.1-.05-1.6-.16L3.3 13.3l.7-2.2C2.7 10.3 2 9 2 7.3 2 4.6 4.6 2.5 8 2.5Z" />
  </Line>
);

export const NoteIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M3.6 2h6l2.8 2.8v8.7c0 .3-.2.5-.5.5H3.6a.5.5 0 0 1-.5-.5V2.5c0-.3.2-.5.5-.5Zm1.6 5h5.6M5.2 9.4h5.6M5.2 11.7h3.6" />
  </Line>
);

export const CursorIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M6 3h4M8 3v10M6 13h4" />
  </Line>
);

export const PlayIcon: React.FC<IconProps> = ({ size = 10, className }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 10 10"
    fill="currentColor"
    className={className}
    aria-hidden="true"
  >
    <path d="M2 1.2v7.6L8.6 5Z" />
  </svg>
);

export const TrashIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M3 4.5h10M6.5 4.5V3h3v1.5M4.5 4.5l.6 8.5h5.8l.6-8.5" />
  </Line>
);

export const RetryIcon: React.FC<IconProps> = (p) => (
  <Line {...p}>
    <path d="M3 8a5 5 0 1 0 1.5-3.6M3 2.5v2.5h2.5" />
  </Line>
);

export const ContextIcon: React.FC<
  IconProps & { context: "chat" | "to_ai" | "notes" | "other" | string }
> = ({ context, ...p }) => {
  switch (context) {
    case "chat":
      return <ChatIcon {...p} />;
    case "to_ai":
      return <SparkleIcon {...p} />;
    case "notes":
      return <NoteIcon {...p} />;
    default:
      return <CursorIcon {...p} />;
  }
};
