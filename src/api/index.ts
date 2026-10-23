// Appels typés vers le cœur Rust (src-tauri/src/commands).
import { Channel, invoke } from "@tauri-apps/api/core";

export type TapeFormat =
  "vhs" | "vhs_c" | "svhs" | "video8" | "hi8" | "digital8" | "minidv" | "file";

export type CassetteStatus = "to_capture" | "captured" | "ready" | "exported";

export interface Settings {
  libraryDir: string | null;
  language: string | null;
}

export interface CassetteSummary {
  id: number;
  uuid: string;
  folder: string;
  title: string;
  format: TapeFormat;
  recordedFrom: string | null;
  status: CassetteStatus;
  durationMs: number | null;
}

export interface FfmpegInfo {
  available: boolean;
  version: string | null;
}

export interface CaptureDevice {
  name: string;
  id: string;
  kind: "video" | "audio";
  dv: boolean;
}

export interface MediaInfo {
  durationMs: number | null;
  width: number | null;
  height: number | null;
  videoCodec: string | null;
  audioCodec: string | null;
  interlaced: boolean;
  playable: boolean;
}

export interface OpenedVideo {
  info: MediaInfo;
  /** Absent tant que la copie de lecture se prépare (événement `proxy-progress`). */
  playPath: string | null;
}

export interface ProxyEvent {
  source: string;
  progress: number;
  playPath: string | null;
  error: string | null;
}

export interface NewCassette {
  title: string;
  format: TapeFormat;
  tvStandard: "pal" | "secam" | "ntsc" | null;
  recordedFrom: string | null;
}

export interface VideoInput {
  pin: number;
  name: string;
  sVideo: boolean;
}

export type CaptureInput =
  | { kind: "directShow"; video: string; audio: string | null; videoPin: number | null }
  | { kind: "avFoundation"; video: string; audio: string | null };

export type SignalState = "unknown" | "ok" | "black" | "snow";

export interface Telemetry {
  elapsedMs: number;
  frames: number;
  dropped: number;
  duplicated: number;
  bytes: number;
  peaksDb: number[];
  signal: SignalState;
  yavg: number | null;
  freeBytes: number | null;
}

export type StopReason = "user" | "endOfTape" | "timer" | "diskFull" | "inputEnded";

export interface CaptureFinished {
  outcome: { reason: StopReason; last: Telemetry | null; fieldOrder: string | null };
  cassette: CassetteSummary | null;
  error: string | null;
}

export interface CaptureSpace {
  freeBytes: number;
  bytesPerHour: number;
  minFreeBytes: number;
}

export const createCassette = (cassette: NewCassette) =>
  invoke<CassetteSummary>("create_cassette", { cassette });

export const listVideoInputs = (device: string) =>
  invoke<VideoInput[]>("list_video_inputs", { device });

export const captureSpace = (cassetteId: number) =>
  invoke<CaptureSpace>("capture_space", { cassetteId });

export const startCapture = (
  request: {
    cassetteId: number;
    input: CaptureInput;
    endOfTapeAfterS: number | null;
    maxDurationS: number | null;
  },
  onPreview: (jpeg: ArrayBuffer) => void,
) => {
  const preview = new Channel<ArrayBuffer>();
  preview.onmessage = onPreview;
  return invoke("start_capture", { request, preview });
};

export const stopCapture = () => invoke("stop_capture");

export const openVideo = (path: string) => invoke<OpenedVideo>("open_video", { path });

export const getSettings = () => invoke<Settings>("get_settings");

export const setLibraryDir = (path: string) => invoke<Settings>("set_library_dir", { path });

export const setLanguage = (language: string | null) =>
  invoke<Settings>("set_language", { language });

export const listCassettes = () => invoke<CassetteSummary[]>("list_cassettes");

export const ffmpegInfo = () => invoke<FfmpegInfo>("ffmpeg_info");

export const listCaptureDevices = () => invoke<CaptureDevice[]>("list_capture_devices");
