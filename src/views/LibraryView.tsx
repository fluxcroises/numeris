import { FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { CassetteSummary } from "../api";
import { filterCassettes, useApp } from "../store/app";
import { formatDuration } from "../format";

export function LibraryView() {
  const { t } = useTranslation();
  const { settings, cassettes, filter, libraryOpen, loadError, pickLibrary } = useApp();

  if (!settings?.libraryDir || !libraryOpen) {
    const missing = Boolean(settings?.libraryDir);
    return (
      <main className="content empty-state">
        <FolderOpen size={40} className="accent-icon" aria-hidden="true" />
        <h1>{missing ? t("library.missingTitle") : t("library.noLibraryTitle")}</h1>
        <p>
          {missing
            ? t("library.missingText", { path: settings?.libraryDir })
            : t("library.noLibraryText")}
        </p>
        {missing && loadError && <p className="error-text">{loadError}</p>}
        <button
          type="button"
          className="button button-primary button-large"
          onClick={() => void pickLibrary()}
        >
          {t("library.chooseFolder")}
        </button>
      </main>
    );
  }

  const shown = filterCassettes(cassettes, filter);
  const heading =
    filter.kind === "all"
      ? t("library.allTitle")
      : filter.kind === "status"
        ? t(`statusFilter.${filter.status}`)
        : t(`format.${filter.format}`);

  return (
    <main className="content">
      <div className="content-header">
        <h1>{heading}</h1>
        <span className="secondary">{t("library.count", { count: shown.length })}</span>
      </div>
      {cassettes.length === 0 ? (
        <div className="empty-state">
          <h2>{t("library.emptyTitle")}</h2>
          <p>{t("library.emptyText")}</p>
        </div>
      ) : (
        <div className="cassette-grid">
          {shown.map((c, i) => (
            <CassetteCard key={c.id} cassette={c} tone={(i % 6) + 1} />
          ))}
        </div>
      )}
    </main>
  );
}

function CassetteCard({ cassette, tone }: { cassette: CassetteSummary; tone: number }) {
  const { t } = useTranslation();
  const year = cassette.recordedFrom?.slice(0, 4) ?? t("library.unknownYear");
  return (
    <article className="cassette-card">
      <div className={`cassette-thumb tone-${tone}`}>
        {cassette.durationMs !== null && (
          <span className="thumb-duration">{formatDuration(cassette.durationMs)}</span>
        )}
      </div>
      <div className="cassette-title">
        <span className={`status-dot status-${cassette.status}`} aria-hidden="true" />
        <h3>{cassette.title}</h3>
      </div>
      <span className="cassette-meta">
        {t(`format.${cassette.format}`)} · {year} · {t(`status.${cassette.status}`)}
      </span>
    </article>
  );
}
