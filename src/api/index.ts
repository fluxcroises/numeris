// Appels typés vers le cœur Rust (src-tauri/src/commands).
import { invoke } from "@tauri-apps/api/core";

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

export const openVideo = (path: string) => invoke<OpenedVideo>("open_video", { path });

export const getSettings = () => invoke<Settings>("get_settings");

export const setLibraryDir = (path: string) => invoke<Settings>("set_library_dir", { path });

export const setLanguage = (language: string | null) =>
  invoke<Settings>("set_language", { language });

export const listCassettes = () => invoke<CassetteSummary[]>("list_cassettes");

export const ffmpegInfo = () => invoke<FfmpegInfo>("ffmpeg_info");

export const listCaptureDevices = () => invoke<CaptureDevice[]>("list_capture_devices");
