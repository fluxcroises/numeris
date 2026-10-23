import { invoke, isTauri } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// L'interface est affichée (après la première image) : le cœur peut le noter,
// et l'essai de lancement de la CI s'arrête là.
if (isTauri()) {
  requestAnimationFrame(() =>
    requestAnimationFrame(() => void invoke("app_ready").catch(() => undefined)),
  );
}
