import { open } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";
import * as api from "../api";
import i18n, { systemLanguage } from "../i18n";

export type View = "library" | "settings" | "player";

/** Filtre de la bibliothèque choisi dans le panneau latéral. */
export type LibraryFilter =
  | { kind: "all" }
  | { kind: "status"; status: api.CassetteStatus }
  | { kind: "format"; format: api.TapeFormat };

interface AppStore {
  view: View;
  filter: LibraryFilter;
  settings: api.Settings | null;
  cassettes: api.CassetteSummary[];
  /** La bibliothèque réglée est-elle ouverte ? Faux si son disque est débranché. */
  libraryOpen: boolean;
  loadError: string | null;
  ffmpeg: api.FfmpegInfo | null;

  setView: (view: View) => void;
  setFilter: (filter: LibraryFilter) => void;
  load: () => Promise<void>;
  reloadCassettes: () => Promise<void>;
  chooseLibrary: (path: string) => Promise<void>;
  /** Demande un dossier à l'utilisateur et en fait la bibliothèque. */
  pickLibrary: () => Promise<void>;
  setLanguage: (language: string | null) => Promise<void>;
}

function applyLanguage(language: string | null) {
  void i18n.changeLanguage(language ?? systemLanguage());
}

export const useApp = create<AppStore>((set, get) => ({
  view: "library",
  filter: { kind: "all" },
  settings: null,
  cassettes: [],
  libraryOpen: false,
  loadError: null,
  ffmpeg: null,

  setView: (view) => set({ view }),
  setFilter: (filter) => set({ filter, view: "library" }),

  load: async () => {
    const settings = await api.getSettings();
    applyLanguage(settings.language);
    set({ settings });
    void api.ffmpegInfo().then((ffmpeg) => set({ ffmpeg }));
    await get().reloadCassettes();
  },

  reloadCassettes: async () => {
    if (!get().settings?.libraryDir) {
      set({ cassettes: [], libraryOpen: false, loadError: null });
      return;
    }
    try {
      const cassettes = await api.listCassettes();
      set({ cassettes, libraryOpen: true, loadError: null });
    } catch (error) {
      // Bibliothèque réglée mais fermée (disque débranché) ou illisible.
      set({ cassettes: [], libraryOpen: false, loadError: String(error) });
    }
  },

  chooseLibrary: async (path) => {
    const settings = await api.setLibraryDir(path);
    set({ settings });
    await get().reloadCassettes();
  },

  pickLibrary: async () => {
    const path = await open({ directory: true });
    if (typeof path === "string") await get().chooseLibrary(path);
  },

  setLanguage: async (language) => {
    const settings = await api.setLanguage(language);
    applyLanguage(settings.language);
    set({ settings });
  },
}));

/** Cassettes retenues par le filtre. */
export function filterCassettes(
  cassettes: api.CassetteSummary[],
  filter: LibraryFilter,
): api.CassetteSummary[] {
  switch (filter.kind) {
    case "all":
      return cassettes;
    case "status":
      return cassettes.filter((c) => c.status === filter.status);
    case "format":
      return cassettes.filter((c) => c.format === filter.format);
  }
}
