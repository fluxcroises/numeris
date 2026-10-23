import { listen } from "@tauri-apps/api/event";
import { Circle, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  type CaptureDevice,
  type CaptureFinished,
  type CaptureSpace,
  type CassetteSummary,
  type TapeFormat,
  type Telemetry,
  type VideoInput,
  captureSpace,
  createCassette,
  listCaptureDevices,
  listVideoInputs,
  startCapture,
  stopCapture,
} from "../api";
import { formatDuration } from "../format";
import { isMac } from "../platform";
import { useApp } from "../store/app";

const FORMATS: TapeFormat[] = ["vhs", "vhs_c", "svhs", "video8", "hi8"];
/** Durée de noir ou de neige après laquelle la capture s'arrête seule. */
const END_OF_TAPE_S = 30;

type Phase = "prepare" | "ready" | "capturing" | "finalizing" | "done";

function gigabytes(bytes: number) {
  return (bytes / 1e9).toFixed(1);
}

/** Bonne source audio par défaut : celle qui porte le nom du boîtier vidéo. */
function matchingAudio(video: CaptureDevice | undefined, audios: CaptureDevice[]) {
  const words = video?.name.split(/\s+/).filter((w) => w.length > 3) ?? [];
  return audios.find((a) => words.some((w) => a.name.includes(w))) ?? audios[0];
}

export function CaptureView() {
  const { t } = useTranslation();
  const { setView, reloadCassettes } = useApp();
  const [phase, setPhase] = useState<Phase>("prepare");
  const [error, setError] = useState<string | null>(null);

  // Préparer
  const [title, setTitle] = useState("");
  const [format, setFormat] = useState<TapeFormat>("vhs");
  const [standard, setStandard] = useState<"pal" | "ntsc">("pal");
  const [cassette, setCassette] = useState<CassetteSummary | null>(null);

  // Brancher
  const [devices, setDevices] = useState<CaptureDevice[]>([]);
  const [videoId, setVideoId] = useState("");
  const [audioId, setAudioId] = useState("");
  const [inputs, setInputs] = useState<VideoInput[]>([]);
  const [pin, setPin] = useState<number | null>(null);
  const [autoStop, setAutoStop] = useState(true);
  const [space, setSpace] = useState<CaptureSpace | null>(null);

  // Capturer
  const [telemetry, setTelemetry] = useState<Telemetry | null>(null);
  const [finalizing, setFinalizing] = useState(0);
  const [finished, setFinished] = useState<CaptureFinished | null>(null);
  const previewUrl = useRef<string | null>(null);
  const [preview, setPreview] = useState<string | null>(null);

  const videos = devices.filter((d) => d.kind === "video" && !d.dv);
  const audios = devices.filter((d) => d.kind === "audio");

  useEffect(() => {
    const stops = [
      listen<Telemetry>("capture-telemetry", (e) => setTelemetry(e.payload)),
      listen<number>("capture-finalizing", (e) => {
        setPhase("finalizing");
        setFinalizing(e.payload);
      }),
      listen<CaptureFinished>("capture-finished", (e) => {
        setFinished(e.payload);
        setPhase("done");
        void reloadCassettes();
      }),
    ];
    return () => {
      stops.forEach((p) => void p.then((stop) => stop()));
      if (previewUrl.current) URL.revokeObjectURL(previewUrl.current);
    };
  }, [reloadCassettes]);

  const prepare = async () => {
    setError(null);
    try {
      const created = await createCassette({
        title,
        format,
        tvStandard: standard,
        recordedFrom: null,
      });
      setCassette(created);
      const [list, free] = await Promise.all([listCaptureDevices(), captureSpace(created.id)]);
      setDevices(list);
      setSpace(free);
      const video = list.find((d) => d.kind === "video" && !d.dv);
      setVideoId(video?.id ?? "");
      setAudioId(
        matchingAudio(
          video,
          list.filter((d) => d.kind === "audio"),
        )?.id ?? "",
      );
      setPhase("ready");
    } catch (e) {
      setError(String(e));
    }
  };

  // Entrées (composite, S-Vidéo) du boîtier choisi ; S-Vidéo de préférence.
  useEffect(() => {
    if (!videoId || isMac) return;
    let current = true;
    listVideoInputs(videoId)
      .then((list) => {
        if (!current) return;
        setInputs(list);
        setPin((list.find((i) => i.sVideo) ?? list[0])?.pin ?? null);
      })
      .catch(() => current && setInputs([]));
    return () => {
      current = false;
    };
  }, [videoId]);

  const start = async () => {
    if (!cassette) return;
    setError(null);
    setTelemetry(null);
    try {
      await startCapture(
        {
          cassetteId: cassette.id,
          input: isMac
            ? { kind: "avFoundation", video: videoId, audio: audioId || null }
            : { kind: "directShow", video: videoId, audio: audioId || null, videoPin: pin },
          endOfTapeAfterS: autoStop ? END_OF_TAPE_S : null,
          maxDurationS: null,
        },
        (jpeg) => {
          const url = URL.createObjectURL(new Blob([jpeg], { type: "image/jpeg" }));
          if (previewUrl.current) URL.revokeObjectURL(previewUrl.current);
          previewUrl.current = url;
          setPreview(url);
        },
      );
      setPhase("capturing");
    } catch (e) {
      setError(String(e));
    }
  };

  const hoursFree =
    space && Math.max(0, (space.freeBytes - space.minFreeBytes) / space.bytesPerHour);

  return (
    <main className="content">
      <div className="content-header">
        <h1>{t("capture.title")}</h1>
        {cassette && <span className="secondary">{cassette.title}</span>}
      </div>
      {error && <p className="player-message error-text">{error}</p>}

      {phase === "prepare" && (
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            void prepare();
          }}
        >
          <label className="form-row">
            <span>{t("capture.cassetteTitle")}</span>
            <input
              className="field-input"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={t("capture.cassetteTitlePlaceholder")}
              required
            />
          </label>
          <label className="form-row">
            <span>{t("capture.format")}</span>
            <select
              className="select"
              value={format}
              onChange={(e) => setFormat(e.target.value as TapeFormat)}
            >
              {FORMATS.map((f) => (
                <option key={f} value={f}>
                  {t(`format.${f}`)}
                </option>
              ))}
            </select>
          </label>
          <label className="form-row">
            <span>{t("capture.standard")}</span>
            <select
              className="select"
              value={standard}
              onChange={(e) => setStandard(e.target.value as "pal" | "ntsc")}
            >
              <option value="pal">{t("capture.pal")}</option>
              <option value="ntsc">{t("capture.ntsc")}</option>
            </select>
          </label>
          <button type="submit" className="button button-primary button-large">
            {t("capture.continue")}
          </button>
        </form>
      )}

      {phase === "ready" && (
        <div className="form">
          <label className="form-row">
            <span>{t("capture.videoSource")}</span>
            <select className="select" value={videoId} onChange={(e) => setVideoId(e.target.value)}>
              {videos.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name}
                </option>
              ))}
            </select>
          </label>
          {inputs.length > 1 && (
            <label className="form-row">
              <span>{t("capture.input")}</span>
              <select
                className="select"
                value={pin ?? ""}
                onChange={(e) => setPin(Number(e.target.value))}
              >
                {inputs.map((i) => (
                  <option key={i.pin} value={i.pin}>
                    {i.sVideo ? t("capture.sVideo") : t("capture.composite")}
                  </option>
                ))}
              </select>
            </label>
          )}
          <label className="form-row">
            <span>{t("capture.audioSource")}</span>
            <select className="select" value={audioId} onChange={(e) => setAudioId(e.target.value)}>
              <option value="">{t("capture.noAudio")}</option>
              {audios.map((d) => (
                <option key={d.id} value={d.id}>
                  {d.name}
                </option>
              ))}
            </select>
          </label>
          <label className="form-row form-check">
            <input
              type="checkbox"
              checked={autoStop}
              onChange={(e) => setAutoStop(e.target.checked)}
            />
            <span>{t("capture.autoStop", { seconds: END_OF_TAPE_S })}</span>
          </label>
          {space && hoursFree !== null && (
            <p className={hoursFree < 3 ? "error-text" : "secondary"}>
              {t("capture.space", {
                free: gigabytes(space.freeBytes),
                hours: hoursFree.toFixed(1),
              })}
            </p>
          )}
          <p className="secondary">{t("capture.hint")}</p>
          <button
            type="button"
            className="button button-primary button-large"
            disabled={!videoId}
            onClick={() => void start()}
          >
            <Circle size={14} fill="currentColor" aria-hidden="true" />
            {t("capture.start")}
          </button>
        </div>
      )}

      {phase === "capturing" && (
        <div className="capture">
          <div className="capture-preview">
            {preview ? (
              <img src={preview} alt={t("capture.previewAlt")} />
            ) : (
              <span className="secondary">{t("capture.waiting")}</span>
            )}
          </div>
          <aside className="capture-panel">
            <span className="capture-counter mono">
              {formatDuration(telemetry?.elapsedMs ?? 0)}
            </span>
            <span className={`signal signal-${telemetry?.signal ?? "unknown"}`}>
              {t(`capture.signal.${telemetry?.signal ?? "unknown"}`)}
            </span>
            <div className="meters" aria-label={t("capture.levels")}>
              {(telemetry?.peaksDb ?? []).map((db, i) => (
                <div key={i} className="meter">
                  <div
                    className={
                      db > -3 ? "meter-bar hot" : db > -12 ? "meter-bar warm" : "meter-bar"
                    }
                    style={{ width: `${Math.max(0, Math.min(100, ((db + 60) / 60) * 100))}%` }}
                  />
                </div>
              ))}
            </div>
            <dl className="capture-stats">
              <dt>{t("capture.size")}</dt>
              <dd className="mono">
                {t("capture.gb", { value: gigabytes(telemetry?.bytes ?? 0) })}
              </dd>
              <dt>{t("capture.dropped")}</dt>
              <dd className={telemetry?.dropped ? "mono error-text" : "mono"}>
                {telemetry?.dropped ?? 0}
              </dd>
              <dt>{t("capture.duplicated")}</dt>
              <dd className="mono">{telemetry?.duplicated ?? 0}</dd>
              {telemetry?.freeBytes != null && (
                <>
                  <dt>{t("capture.free")}</dt>
                  <dd className="mono">
                    {t("capture.gb", { value: gigabytes(telemetry.freeBytes) })}
                  </dd>
                </>
              )}
            </dl>
            <button
              type="button"
              className="button button-large"
              onClick={() => void stopCapture()}
            >
              <Square size={13} fill="currentColor" aria-hidden="true" />
              {t("capture.stop")}
            </button>
          </aside>
        </div>
      )}

      {phase === "finalizing" && (
        <div className="empty-state">
          <p>{t("capture.finalizing", { percent: Math.round(finalizing * 100) })}</p>
          <progress className="progress" value={finalizing} max={1} />
        </div>
      )}

      {phase === "done" && finished && (
        <div className="empty-state">
          <h2>{finished.cassette ? t("capture.doneTitle") : t("capture.failedTitle")}</h2>
          <p>{t(`capture.reason.${finished.outcome.reason}`)}</p>
          {finished.cassette?.durationMs != null && (
            <p className="mono">{formatDuration(finished.cassette.durationMs)}</p>
          )}
          {finished.error && <p className="error-text">{finished.error}</p>}
          <button
            type="button"
            className="button button-primary"
            onClick={() => setView("library")}
          >
            {t("capture.backToLibrary")}
          </button>
        </div>
      )}
    </main>
  );
}
