// Yuyin fork: small building blocks for the 1.0 window (design:
// https://claude.ai/artifact/CcyDKSUUKAkAwsPq5siWFA). Apple-style grouped
// lists, switches and segmented controls on the shared color tokens.
import React from "react";

export const Card: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className = "" }) => (
  <div
    className={`bg-surface rounded-[14px] shadow-[0_0_0_0.5px_var(--color-hairline),0_1px_3px_rgba(0,0,0,0.04)] ${className}`}
  >
    {children}
  </div>
);

export const PageTitle: React.FC<{
  title: string;
  subtitle?: string;
  right?: React.ReactNode;
}> = ({ title, subtitle, right }) => (
  <div className="flex items-end justify-between gap-6">
    <div className="flex flex-col gap-1 min-w-0">
      <h1 className="m-0 text-[26px] font-bold tracking-[-0.02em] leading-tight text-text">
        {title}
      </h1>
      {subtitle && (
        <p className="m-0 text-[12px] leading-relaxed text-muted">{subtitle}</p>
      )}
    </div>
    {right}
  </div>
);

/** A titled inset grouped list; rows are divided by hairlines. */
export const Group: React.FC<{
  title?: string;
  footnote?: React.ReactNode;
  children: React.ReactNode;
}> = ({ title, footnote, children }) => (
  <section className="flex flex-col gap-1.5">
    {title && (
      <h2 className="m-0 ps-1 text-[12px] font-semibold text-muted">{title}</h2>
    )}
    <div className="bg-surface rounded-[12px] shadow-[0_0_0_0.5px_var(--color-hairline)] divide-y divide-hairline">
      {children}
    </div>
    {footnote && (
      <p className="m-0 px-1 text-[11px] leading-relaxed text-muted">
        {footnote}
      </p>
    )}
  </section>
);

export const Row: React.FC<{
  label: React.ReactNode;
  description?: React.ReactNode;
  htmlFor?: string;
  children?: React.ReactNode;
}> = ({ label, description, htmlFor, children }) => (
  <div className="flex items-center justify-between gap-4 min-h-11 px-3.5 py-2">
    <div className="flex flex-col gap-0.5 min-w-0">
      {htmlFor ? (
        <label htmlFor={htmlFor} className="text-[13px] text-text">
          {label}
        </label>
      ) : (
        <span className="text-[13px] text-text">{label}</span>
      )}
      {description && (
        <span className="text-[11px] leading-snug text-muted">
          {description}
        </span>
      )}
    </div>
    {children && (
      <div className="flex items-center gap-2 shrink-0">{children}</div>
    )}
  </div>
);

export const Switch: React.FC<{
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;
  disabled?: boolean;
}> = ({ checked, onChange, label, disabled }) => (
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    disabled={disabled}
    onClick={() => onChange(!checked)}
    className={`w-[38px] h-[22px] rounded-full p-[2px] flex transition-colors duration-200 disabled:opacity-40 ${
      checked ? "bg-[#34c759] justify-end" : "bg-black/15 dark:bg-white/20"
    }`}
  >
    <span className="w-[18px] h-[18px] rounded-full bg-white shadow-[0_1px_2px_rgba(0,0,0,0.25)]" />
  </button>
);

export function Segmented<T extends string>({
  options,
  value,
  onChange,
  label,
  disabled,
}: {
  options: { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
  label: string;
  disabled?: boolean;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={`grid gap-[2px] p-[2px] rounded-[8px] bg-black/[0.06] dark:bg-white/10 ${disabled ? "opacity-50" : ""}`}
      style={{
        gridTemplateColumns: `repeat(${options.length}, minmax(0, 1fr))`,
      }}
    >
      {options.map((o) => {
        const on = o.value === value;
        return (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={on}
            disabled={disabled}
            onClick={() => onChange(o.value)}
            className={`text-[13px] py-[5px] rounded-[7px] transition-colors ${
              on
                ? "bg-surface font-semibold text-text shadow-[0_0_0_0.5px_rgba(0,0,0,0.1),0_1px_2px_rgba(0,0,0,0.12)]"
                : "text-text/80"
            }`}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

export const Kbd: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <kbd className="font-sans text-[11px] leading-none px-[7px] py-[4px] rounded-[6px] bg-background text-text shadow-[0_0_0_0.5px_rgba(0,0,0,0.16),0_1px_0_rgba(0,0,0,0.08)] dark:shadow-[0_0_0_0.5px_rgba(255,255,255,0.16)]">
    {children}
  </kbd>
);

export const SmallButton: React.FC<
  React.ButtonHTMLAttributes<HTMLButtonElement>
> = ({ className = "", children, ...rest }) => (
  <button
    type="button"
    className={`text-[12px] px-2.5 py-[4px] rounded-[7px] bg-fill text-text hover:brightness-95 active:brightness-90 disabled:opacity-40 transition ${className}`}
    {...rest}
  >
    {children}
  </button>
);

export const PrimaryButton: React.FC<
  React.ButtonHTMLAttributes<HTMLButtonElement>
> = ({ className = "", children, ...rest }) => (
  <button
    type="button"
    className={`min-w-[140px] px-6 py-[7px] rounded-full bg-logo-primary text-white text-[14px] font-medium hover:brightness-110 active:brightness-95 disabled:opacity-40 transition focus:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary/40 focus-visible:ring-offset-2 ${className}`}
    {...rest}
  >
    {children}
  </button>
);

export const TextInput = React.forwardRef<
  HTMLInputElement,
  React.InputHTMLAttributes<HTMLInputElement>
>(({ className = "", ...rest }, ref) => (
  <input
    ref={ref}
    className={`text-[13px] px-3 py-[7px] rounded-[9px] bg-surface text-text shadow-[0_0_0_0.5px_rgba(0,0,0,0.16)] dark:shadow-[0_0_0_0.5px_rgba(255,255,255,0.16)] outline-none focus:shadow-[0_0_0_2px_var(--color-logo-primary)] placeholder:text-muted ${className}`}
    {...rest}
  />
));
TextInput.displayName = "TextInput";

export const Select: React.FC<
  React.SelectHTMLAttributes<HTMLSelectElement>
> = ({ className = "", children, ...rest }) => (
  <select
    className={`text-[13px] ps-2 pe-7 py-[4px] rounded-[7px] bg-fill text-text outline-none focus-visible:shadow-[0_0_0_2px_var(--color-logo-primary)] ${className}`}
    {...rest}
  >
    {children}
  </select>
);
