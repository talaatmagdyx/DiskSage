import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./app/App";
import "./styles.css";

async function mount() {
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).get("release-preview")) {
    const { bootstrapPreviewScenario } = await import("./ipc/releasePreview");
    await bootstrapPreviewScenario();
  }

  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void mount();
