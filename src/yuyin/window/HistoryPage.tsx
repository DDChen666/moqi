// Yuyin fork: history as the design shows it — grouped by day, each entry
// with where it was typed, what was pasted, and (expanded) the original
// words, the recording, and exactly what left the machine.
import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { convertFileSrc } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";
import { commands, events, type HistoryEntry } from "@/bindings";
import { useOsType } from "@/hooks/useOsType";
import { AudioPlayer, AudioPlayerGroup } from "@/components/ui/AudioPlayer";
import { yuyinApi, type EntryMeta } from "../api";
import { PageTitle, SmallButton } from "./ui";
import {
  ContextIcon,
  LockIcon,
  RetryIcon,
  SearchIcon,
  TrashIcon,
} from "./icons";
import { dayLabel, secondsLabel, timeLabel } from "./format";
import { serviceName } from "./HomePage";

const PAGE_SIZE = 50;

const finalText = (e: HistoryEntry) =>
  e.post_processed_text ?? e.transcription_text;

const EntryRow: React.FC<{
  entry: HistoryEntry;
  meta?: EntryMeta;
  open: boolean;
  onToggle: () => void;
  getAudioUrl: (fileName: string) => Promise<string | null>;
  onDelete: (id: number) => void;
}> = ({ entry, meta, open, onToggle, getAudioUrl, onDelete }) => {
  const { t, i18n } = useTranslation();
  const [retrying, setRetrying] = useState(false);
  const text = finalText(entry);
  const failed = text.trim().length === 0;
  const local = meta ? meta.sent_chars === 0 : false;
  const levelName = meta?.level ? t(`moqi.levels.${meta.level}`) : null;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(t("moqi.history.copied"));
    } catch {
      toast.error(t("settings.history.copyError"));
    }
  };
  const repaste = async () => {
    try {
      await yuyinApi.repaste(text);
    } catch (e) {
      console.error("Repaste failed:", e);
      toast.error(t("moqi.history.repasteFailed"));
    }
  };
  const retry = async () => {
    setRetrying(true);
    try {
      const result = await commands.retryHistoryEntryTranscription(entry.id);
      if (result.status !== "ok") throw new Error(String(result.error));
    } catch (e) {
      console.error("Retry failed:", e);
      toast.error(t("settings.history.retranscribeError"));
    } finally {
      setRetrying(false);
    }
  };

  return (
    <div className="bg-surface rounded-[12px] shadow-[0_0_0_0.5px_var(--color-hairline),0_1px_3px_rgba(0,0,0,0.04)] flex flex-col">
      <div className="flex items-start gap-3 px-3.5 py-3">
        <button
          type="button"
          onClick={onToggle}
          aria-expanded={open}
          className="flex-1 min-w-0 text-start flex flex-col gap-1.5 cursor-pointer"
        >
          <span className="flex items-center gap-2 flex-wrap">
            <span className="text-[11px] text-muted tabular-nums">
              {timeLabel(entry.timestamp, i18n.language)}
            </span>
            {meta?.app && (
              <span className="inline-flex items-center gap-1 text-[11px] px-[7px] py-[2px] rounded-[6px] bg-fill text-text/80">
                <ContextIcon context={meta.context} size={11} />
                {meta.app}
              </span>
            )}
            {levelName && (
              <span className="text-[11px] text-muted">{levelName}</span>
            )}
            {local && (
              <span className="inline-flex items-center gap-[3px] text-[11px] text-positive">
                <LockIcon size={10} />
                {t("moqi.history.allLocal")}
              </span>
            )}
          </span>
          <span
            className={`text-[14px] leading-[1.55] whitespace-pre-line break-words ${
              failed ? "text-muted italic" : "text-text select-text"
            }`}
          >
            {retrying
              ? t("settings.history.transcribing")
              : failed
                ? t("moqi.history.failed")
                : text}
          </span>
        </button>
        <div className="flex gap-1.5 shrink-0">
          {failed ? (
            <SmallButton
              onClick={retry}
              disabled={retrying}
              className="inline-flex items-center gap-1"
            >
              <RetryIcon size={12} />
              {t("moqi.history.retry")}
            </SmallButton>
          ) : (
            <>
              <SmallButton onClick={copy}>{t("moqi.history.copy")}</SmallButton>
              <SmallButton
                onClick={repaste}
                title={t("moqi.history.repasteHint")}
              >
                {t("moqi.history.repaste")}
              </SmallButton>
            </>
          )}
        </div>
      </div>

      {open && (
        <div className="border-t border-hairline px-3.5 pt-3 pb-3.5 grid grid-cols-2 gap-[18px]">
          <div className="flex flex-col gap-1.5 min-w-0">
            <div className="text-[11px] font-semibold text-muted">
              {t("moqi.history.original")}
            </div>
            <div className="text-[12.5px] leading-relaxed text-text/80 select-text whitespace-pre-line break-words">
              {entry.transcription_text || "—"}
            </div>
            <AudioPlayer
              onLoadRequest={() => getAudioUrl(entry.file_name)}
              className="w-full mt-1"
            />
            {meta?.spoke_ms != null && (
              <div className="text-[11px] text-muted">
                {meta.output_ms != null
                  ? t("moqi.history.timing", {
                      spoke: secondsLabel(meta.spoke_ms, t),
                      output: secondsLabel(meta.output_ms, t),
                    })
                  : t("moqi.history.spoke", {
                      spoke: secondsLabel(meta.spoke_ms, t),
                    })}
              </div>
            )}
          </div>
          <div className="flex flex-col gap-[7px] min-w-0">
            <div className="text-[11px] font-semibold text-muted">
              {t("moqi.history.whatWasSent")}
            </div>
            {!meta ? (
              <div className="text-[12.5px] text-muted">
                {t("moqi.history.noRecord")}
              </div>
            ) : local ? (
              <div className="text-[12.5px] text-positive">
                {t("moqi.history.nothingSent")}
              </div>
            ) : (
              <>
                <div className="flex gap-[7px] items-baseline text-[12.5px] text-text">
                  <span className="text-logo-primary font-semibold shrink-0">
                    {t("moqi.history.sent")}
                  </span>
                  <span>
                    {t("moqi.history.sentText", {
                      count: meta.sent_chars,
                      to: meta.sent_to ? serviceName(meta.sent_to) : "",
                    })}
                  </span>
                </div>
                <div className="flex gap-[7px] items-baseline text-[12.5px] text-text">
                  <span className="text-positive font-semibold shrink-0">
                    {t("moqi.history.notSent")}
                  </span>
                  <span>{t("moqi.history.audio")}</span>
                </div>
                <div className="flex gap-[7px] items-baseline text-[12.5px] text-text">
                  <span className="text-positive font-semibold shrink-0">
                    {t("moqi.history.notSent")}
                  </span>
                  <span>
                    {t("moqi.history.appAndTitle", {
                      style: t(`moqi.styles.${meta.context}`),
                    })}
                  </span>
                </div>
              </>
            )}
            <button
              type="button"
              onClick={() => onDelete(entry.id)}
              className="self-start mt-auto inline-flex items-center gap-1 text-[11px] text-muted hover:text-error"
            >
              <TrashIcon size={12} />
              {t("moqi.history.delete")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export const HistoryPage: React.FC = () => {
  const { t, i18n } = useTranslation();
  const os = useOsType();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [meta, setMeta] = useState<Record<string, EntryMeta>>({});
  const [hasMore, setHasMore] = useState(false);
  const [loading, setLoading] = useState(true);
  const [openId, setOpenId] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const loadingRef = useRef(false);

  const loadMeta = useCallback(() => {
    yuyinApi
      .historyMeta()
      .then(setMeta)
      .catch((e) => console.error("Failed to load history details:", e));
  }, []);

  const loadPage = useCallback(async (cursor?: number) => {
    if (loadingRef.current) return;
    loadingRef.current = true;
    try {
      const result = await commands.getHistoryEntries(
        cursor ?? null,
        PAGE_SIZE,
      );
      if (result.status === "ok") {
        setEntries((prev) =>
          cursor === undefined
            ? result.data.entries
            : [...prev, ...result.data.entries],
        );
        setHasMore(result.data.has_more);
      }
    } finally {
      setLoading(false);
      loadingRef.current = false;
    }
  }, []);

  useEffect(() => {
    loadPage();
    loadMeta();
    const unlisten = events.historyUpdatePayload.listen((event) => {
      const p = event.payload;
      if (p.action === "added") {
        setEntries((prev) => [p.entry, ...prev]);
        loadMeta();
      } else if (p.action === "updated") {
        setEntries((prev) =>
          prev.map((e) => (e.id === p.entry.id ? p.entry : e)),
        );
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [loadPage, loadMeta]);

  const getAudioUrl = useCallback(
    async (fileName: string) => {
      const result = await commands.getAudioFilePath(fileName);
      if (result.status !== "ok") return null;
      if (os === "linux") {
        const data = await readFile(result.data);
        return URL.createObjectURL(new Blob([data], { type: "audio/wav" }));
      }
      return convertFileSrc(result.data, "asset");
    },
    [os],
  );

  const remove = async (id: number) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
    const result = await commands.deleteHistoryEntry(id);
    if (result.status !== "ok") loadPage();
  };

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase();
    const shown = q
      ? entries.filter(
          (e) =>
            finalText(e).toLowerCase().includes(q) ||
            e.transcription_text.toLowerCase().includes(q) ||
            (meta[e.file_name]?.app ?? "").toLowerCase().includes(q),
        )
      : entries;
    const out: { label: string; items: HistoryEntry[] }[] = [];
    for (const e of shown) {
      const label = dayLabel(e.timestamp, i18n.language, t);
      const last = out[out.length - 1];
      if (last && last.label === label) last.items.push(e);
      else out.push({ label, items: [e] });
    }
    return out;
  }, [entries, meta, query, i18n.language, t]);

  return (
    <div className="flex flex-col gap-[18px]">
      <PageTitle
        title={t("moqi.nav.history")}
        subtitle={t("moqi.history.subtitle")}
        right={
          <label className="flex items-center gap-1.5 w-[200px] px-2.5 py-1.5 rounded-[8px] bg-surface shadow-[0_0_0_0.5px_rgba(0,0,0,0.12)] dark:shadow-[0_0_0_0.5px_rgba(255,255,255,0.14)]">
            <SearchIcon size={13} className="text-muted shrink-0" />
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("moqi.history.search")}
              aria-label={t("moqi.history.search")}
              className="w-full bg-transparent outline-none text-[12px] text-text placeholder:text-muted"
            />
          </label>
        }
      />

      {!loading && entries.length === 0 && (
        <p className="text-[13px] text-muted">{t("moqi.history.empty")}</p>
      )}

      <AudioPlayerGroup>
        <div className="flex flex-col gap-4">
          {groups.map((g) => (
            <div key={g.label} className="flex flex-col gap-2">
              <div className="text-[11px] font-semibold text-muted ps-0.5">
                {g.label}
              </div>
              {g.items.map((e) => (
                <EntryRow
                  key={e.id}
                  entry={e}
                  meta={meta[e.file_name]}
                  open={openId === e.id}
                  onToggle={() =>
                    setOpenId((id) => (id === e.id ? null : e.id))
                  }
                  getAudioUrl={getAudioUrl}
                  onDelete={remove}
                />
              ))}
            </div>
          ))}
        </div>
      </AudioPlayerGroup>

      {hasMore && (
        <SmallButton
          className="self-center"
          onClick={() => loadPage(entries[entries.length - 1]?.id)}
        >
          {t("moqi.history.more")}
        </SmallButton>
      )}
    </div>
  );
};
