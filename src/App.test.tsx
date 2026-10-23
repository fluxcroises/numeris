import { render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";
import type { CassetteSummary, Settings } from "./api";

// Le cœur Rust est simulé : chaque commande renvoie ce que le test prépare.
const backend: { settings: Settings; cassettes: CassetteSummary[] } = {
  settings: { libraryDir: null, language: "fr" },
  cassettes: [],
};

vi.mock("@tauri-apps/api/core", () => ({
  isTauri: () => false,
  invoke: vi.fn(async (command: string) => {
    switch (command) {
      case "get_settings":
        return backend.settings;
      case "list_cassettes":
        return backend.cassettes;
      case "ffmpeg_info":
        return { available: true, version: "9.0.2" };
      case "list_capture_devices":
        return [];
      default:
        throw new Error(`commande inattendue : ${command}`);
    }
  }),
}));

const { default: App } = await import("./App");
const { useApp } = await import("./store/app");

beforeEach(() => {
  backend.settings = { libraryDir: null, language: "fr" };
  backend.cassettes = [];
  useApp.setState({ view: "library", filter: { kind: "all" }, cassettes: [], settings: null });
});

test("sans bibliothèque, propose de choisir un dossier", async () => {
  render(<App />);
  expect(await screen.findByText("Où ranger vos cassettes ?")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Choisir un dossier" })).toBeInTheDocument();
});

test("affiche les cassettes et leurs compteurs", async () => {
  backend.settings = { libraryDir: "D:\\Numeris", language: "fr" };
  backend.cassettes = [
    {
      id: 1,
      uuid: "u1",
      folder: "1998-noel",
      title: "Noël chez Mamie",
      format: "vhs",
      recordedFrom: "1998-12",
      status: "ready",
      durationMs: 10_030_000,
    },
  ];
  render(<App />);
  expect(await screen.findByText("Noël chez Mamie")).toBeInTheDocument();
  expect(screen.getByText("2:47:10")).toBeInTheDocument();
  expect(screen.getByText("VHS · 1998 · Prête")).toBeInTheDocument();
  expect(screen.getByText("1 cassette")).toBeInTheDocument();
});

test("la barre de titre a ses boutons de fenêtre accessibles", async () => {
  render(<App />);
  await screen.findByText("Où ranger vos cassettes ?");
  expect(screen.getByRole("button", { name: "Fermer" })).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Réglages" })).toBeInTheDocument();
});
