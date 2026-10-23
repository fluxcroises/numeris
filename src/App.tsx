import { useEffect } from "react";
import "./App.css";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { TitleBar } from "./components/TitleBar";
import { useApp } from "./store/app";
import { LibraryView } from "./views/LibraryView";
import { PlayerView } from "./views/PlayerView";
import { SettingsView } from "./views/SettingsView";

export default function App() {
  const view = useApp((s) => s.view);
  const load = useApp((s) => s.load);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <div className="app">
      <TitleBar />
      <div className="app-body">
        <Sidebar />
        {view === "settings" ? (
          <SettingsView />
        ) : view === "player" ? (
          <PlayerView />
        ) : (
          <LibraryView />
        )}
      </div>
      <StatusBar />
    </div>
  );
}
