import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import init from "wasm";
import App from "./App";

init().then(() => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
