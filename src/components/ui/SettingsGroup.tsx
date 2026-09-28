import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  return (
    // Yuyin fork: an inset grouped list, as in System Settings.
    <div className="space-y-1.5">
      {title && (
        <div className="px-2.5">
          <h2 className="text-[13px] font-semibold text-text/85">{title}</h2>
          {description && (
            <p className="text-xs text-mid-gray mt-0.5">{description}</p>
          )}
        </div>
      )}
      <div className="bg-surface rounded-[10px] overflow-visible shadow-[0_0_0_0.5px_var(--color-hairline),0_1px_2px_rgba(0,0,0,0.04)]">
        <div className="divide-y divide-hairline">{children}</div>
      </div>
    </div>
  );
};
