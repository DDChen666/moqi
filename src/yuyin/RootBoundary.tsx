// Yuyin fork: the last line of defence around the whole window. A render
// error used to unmount everything and leave a blank window with no trace;
// now it is written to the app log and the window offers a reload.
import { Component, type ErrorInfo, type ReactNode } from "react";
import i18n from "@/i18n";
import { yuyinApi } from "./api";

const describe = (error: unknown) =>
  error instanceof Error ? `${error.name}: ${error.message}` : String(error);

if (typeof window !== "undefined") {
  window.addEventListener("error", (e) =>
    yuyinApi.reportError(
      `${describe(e.error ?? e.message)} at ${e.filename}:${e.lineno}`,
    ),
  );
  window.addEventListener("unhandledrejection", (e) =>
    yuyinApi.reportError(`unhandled rejection: ${describe(e.reason)}`),
  );
}

export class RootBoundary extends Component<
  { children: ReactNode },
  { failed: boolean }
> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    yuyinApi.reportError(
      `${describe(error)}\n${error.stack ?? ""}\ncomponents:${info.componentStack ?? ""}`,
    );
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <div className="h-screen w-full flex flex-col items-center justify-center gap-4 bg-background text-text">
        <div
          data-tauri-drag-region
          className="fixed top-0 inset-x-0 h-[52px]"
        />
        <p className="text-[15px] font-semibold">
          {i18n.t("errors.windowCrashed")}
        </p>
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="px-5 py-[6px] rounded-full bg-logo-primary text-white text-[13px] font-medium"
        >
          {i18n.t("errors.reload")}
        </button>
      </div>
    );
  }
}
