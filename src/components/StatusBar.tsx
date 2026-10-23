import { useTranslation } from "react-i18next";
import { useApp } from "../store/app";

export function StatusBar() {
  const { t } = useTranslation();
  const { settings, libraryOpen, ffmpeg } = useApp();

  return (
    <footer className="statusbar">
      <span>
        {settings?.libraryDir && libraryOpen
          ? t("statusBar.library", { path: settings.libraryDir })
          : t("statusBar.noLibrary")}
      </span>
      <span className="statusbar-spacer" />
      {ffmpeg?.version && <span>{t("statusBar.ffmpeg", { version: ffmpeg.version })}</span>}
    </footer>
  );
}
