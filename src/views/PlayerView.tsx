import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { FileVideo } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type MediaInfo, type ProxyEvent, openVideo } from "../api";
import { formatDuration } from "../format";

const VIDEO_EXTENSIONS = ["mp4", "mov", "mkv", "avi", "dv", "vob", "mpg", "mpeg", "m4v"];

interface Opened {
  source: string;
  info: MediaInfo;
  playPath: string | null;
  progress: number;
}

export function PlayerView() {
  const { t } = useTranslation();
  const [video, setVideo] = useState<Opened | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Avancement de la copie de lecture préparée par le cœur.
  useEffect(() => {
    const unlisten = listen<ProxyEvent>("proxy-progress", ({ payload }) => {
      setVideo((current) => {
        if (!current || current.source !== payload.source) return current;
        return { ...current, progress: payload.progress, playPath: payload.playPath };
      });
      if (payload.error) setError(payload.error);
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  const choose = async () => {
    const path = await open({
      filters: [{ name: t("player.videoFiles"), extensions: VIDEO_EXTENSIONS }],
    });
    if (typeof path !== "string") return;
    setError(null);
    setVideo(null);
    try {
      const opened = await openVideo(path);
      setVideo({ source: path, info: opened.info, playPath: opened.playPath, progress: 0 });
    } catch (e) {
      setError(String(e));
    }
  };

  const info = video?.info;
  return (
    <main className="content">
      <div className="content-header">
        <h1>{t("player.title")}</h1>
        {info && (
          <span className="secondary">
            {t("player.info", {
              width: info.width ?? "?",
              height: info.height ?? "?",
              codec: info.videoCodec ?? "?",
              duration: info.durationMs === null ? "?" : formatDuration(info.durationMs),
            })}
            {info.interlaced && ` · ${t("player.interlaced")}`}
          </span>
        )}
        <span className="statusbar-spacer" />
        <button type="button" className="button" onClick={() => void choose()}>
          <FileVideo size={14} aria-hidden="true" />
          {t("player.open")}
        </button>
      </div>
      {error && <p className="player-message error-text">{t("player.error", { error })}</p>}
      {!video && !error && (
        <div className="empty-state">
          <FileVideo size={40} className="accent-icon" aria-hidden="true" />
          <p>{t("player.empty")}</p>
        </div>
      )}
      {video && !video.playPath && !error && (
        <div className="empty-state">
          <p>{t("player.preparing", { percent: Math.round(video.progress * 100) })}</p>
          <progress className="progress" value={video.progress} max={1} />
        </div>
      )}
      {video?.playPath && (
        <div className="player">
          <video key={video.playPath} src={convertFileSrc(video.playPath)} controls autoPlay />
        </div>
      )}
    </main>
  );
}
