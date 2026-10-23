import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Circle, Minus, Search, SlidersHorizontal, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { isMac } from "../platform";
import { useApp } from "../store/app";
import { Logo } from "./Logo";

/** Barre de titre intégrée : la fenêtre n'a pas de cadre système sous Windows. */
export function TitleBar() {
  const { t } = useTranslation();
  const setView = useApp((s) => s.setView);
  const view = useApp((s) => s.view);
  const win = isTauri() ? getCurrentWindow() : null;

  return (
    <header className={isMac ? "titlebar titlebar-mac" : "titlebar"} data-tauri-drag-region>
      <div className="titlebar-brand" data-tauri-drag-region>
        <Logo size={24} />
        <span className="titlebar-name">{t("app.name")}</span>
      </div>
      <label className="field titlebar-search">
        <Search size={14} aria-hidden="true" />
        <span className="visually-hidden">{t("titleBar.search")}</span>
        <input type="search" placeholder={t("titleBar.search")} />
      </label>
      <div className="titlebar-spacer" data-tauri-drag-region />
      <button
        type="button"
        className="button button-primary"
        disabled
        title={t("titleBar.digitizeSoon")}
      >
        <Circle size={12} fill="currentColor" aria-hidden="true" />
        {t("titleBar.digitize")}
      </button>
      <button
        type="button"
        className={view === "settings" ? "icon-button active" : "icon-button"}
        aria-label={t("titleBar.settings")}
        aria-pressed={view === "settings"}
        onClick={() => setView(view === "settings" ? "library" : "settings")}
      >
        <SlidersHorizontal size={16} aria-hidden="true" />
      </button>
      {!isMac && (
        <div className="window-controls">
          <button
            type="button"
            aria-label={t("titleBar.minimize")}
            onClick={() => void win?.minimize()}
          >
            <Minus size={14} aria-hidden="true" />
          </button>
          <button
            type="button"
            aria-label={t("titleBar.maximize")}
            onClick={() => void win?.toggleMaximize()}
          >
            <Square size={11} aria-hidden="true" />
          </button>
          <button
            type="button"
            className="window-close"
            aria-label={t("titleBar.close")}
            onClick={() => void win?.close()}
          >
            <X size={15} aria-hidden="true" />
          </button>
        </div>
      )}
    </header>
  );
}
