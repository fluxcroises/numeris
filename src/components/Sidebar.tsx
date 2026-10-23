import { LayoutGrid, PlayCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CassetteStatus, TapeFormat } from "../api";
import { type LibraryFilter, useApp } from "../store/app";

const STATUSES: CassetteStatus[] = ["to_capture", "captured", "ready", "exported"];
const FORMATS: TapeFormat[] = [
  "vhs",
  "vhs_c",
  "svhs",
  "video8",
  "hi8",
  "digital8",
  "minidv",
  "file",
];

function same(a: LibraryFilter, b: LibraryFilter) {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function Sidebar() {
  const { t } = useTranslation();
  const { cassettes, filter, setFilter, view, setView } = useApp();

  const item = (target: LibraryFilter, label: string, count: number, marker?: React.ReactNode) => {
    const current = view === "library" && same(filter, target);
    return (
      <button
        type="button"
        className={current ? "sidebar-item current" : "sidebar-item"}
        aria-current={current ? "page" : undefined}
        onClick={() => setFilter(target)}
      >
        {marker}
        <span className="sidebar-label">{label}</span>
        <span className="sidebar-count">{count}</span>
      </button>
    );
  };

  // Seuls les supports présents dans la bibliothèque sont proposés.
  const formats = FORMATS.filter((f) => cassettes.some((c) => c.format === f));

  return (
    <nav className="sidebar" aria-label={t("sidebar.label")}>
      <h2 className="section-title">{t("sidebar.cassettes")}</h2>
      {item(
        { kind: "all" },
        t("sidebar.all"),
        cassettes.length,
        <LayoutGrid size={14} className="accent-icon" aria-hidden="true" />,
      )}
      {STATUSES.map((status) =>
        item(
          { kind: "status", status },
          t(`statusFilter.${status}`),
          cassettes.filter((c) => c.status === status).length,
          <span className={`status-dot status-${status}`} aria-hidden="true" />,
        ),
      )}

      {formats.length > 0 && <h2 className="section-title">{t("sidebar.formats")}</h2>}
      {formats.map((format) =>
        item(
          { kind: "format", format },
          t(`format.${format}`),
          cassettes.filter((c) => c.format === format).length,
        ),
      )}

      <h2 className="section-title">{t("sidebar.tools")}</h2>
      <button
        type="button"
        className={view === "player" ? "sidebar-item current" : "sidebar-item"}
        aria-current={view === "player" ? "page" : undefined}
        onClick={() => setView("player")}
      >
        <PlayCircle size={14} aria-hidden="true" />
        <span className="sidebar-label">{t("sidebar.player")}</span>
      </button>
    </nav>
  );
}
