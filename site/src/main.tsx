import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./estilo.css";
import "./componentes.css";

createRoot(document.getElementById("raiz")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
