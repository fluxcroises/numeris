import { Camera, Mic, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { type CaptureDevice, listCaptureDevices } from "../api";
import { useApp } from "../store/app";

export function SettingsView() {
  const { t } = useTranslation();
  const { settings, ffmpeg, pickLibrary, setLanguage } = useApp();
  const [devices, setDevices] = useState<CaptureDevice[] | null>(null);
  const [devicesError, setDevicesError] = useState<string | null>(null);

  // Chaque clic sur « Actualiser » relance la détection.
  const [round, setRound] = useState(0);

  useEffect(() => {
    let current = true;
    listCaptureDevices()
      .then((list) => {
        if (!current) return;
        setDevices(list);
        setDevicesError(null);
      })
      .catch((error) => {
        if (!current) return;
        setDevices([]);
        setDevicesError(String(error));
      });
    return () => {
      current = false;
    };
  }, [round]);

  const refresh = () => {
    setDevices(null);
    setRound((n) => n + 1);
  };

  return (
    <main className="content settings">
      <div className="content-header">
        <h1>{t("settings.title")}</h1>
      </div>

      <section className="settings-section">
        <h2 className="section-title">{t("settings.library")}</h2>
        <div className="settings-row">
          <span className="settings-label">{t("settings.libraryFolder")}</span>
          <span className="settings-value">{settings?.libraryDir ?? t("settings.noFolder")}</span>
          <button type="button" className="button" onClick={() => void pickLibrary()}>
            {t("settings.change")}
          </button>
        </div>
        <label className="settings-row">
          <span className="settings-label">{t("settings.language")}</span>
          <select
            className="select"
            value={settings?.language ?? ""}
            onChange={(e) => void setLanguage(e.target.value || null)}
          >
            <option value="">{t("settings.languageSystem")}</option>
            <option value="fr">Français</option>
            <option value="en">English</option>
          </select>
        </label>
      </section>

      <section className="settings-section">
        <h2 className="section-title">{t("settings.hardware")}</h2>
        <div className="settings-row">
          <span className="settings-label">{t("settings.ffmpeg")}</span>
          <span className="settings-value mono">
            {ffmpeg === null ? "…" : (ffmpeg.version ?? t("settings.ffmpegMissing"))}
          </span>
        </div>
        <div className="settings-row">
          <span className="settings-label">{t("settings.devices")}</span>
          <span className="settings-value" />
          <button type="button" className="button" onClick={refresh}>
            <RefreshCw size={13} aria-hidden="true" />
            {t("settings.refresh")}
          </button>
        </div>
        {devicesError && <p className="error-text">{devicesError}</p>}
        {devices?.length === 0 && !devicesError && (
          <p className="secondary">{t("settings.noDevices")}</p>
        )}
        <ul className="device-list">
          {devices?.map((d) => (
            <li key={`${d.kind}-${d.id}`}>
              {d.kind === "video" ? (
                <Camera size={14} aria-hidden="true" />
              ) : (
                <Mic size={14} aria-hidden="true" />
              )}
              <span className="device-name">{d.name}</span>
              <span className="secondary">{d.dv ? t("settings.dv") : t(`settings.${d.kind}`)}</span>
            </li>
          ))}
        </ul>
      </section>
    </main>
  );
}
