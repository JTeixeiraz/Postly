import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { ProvedorIdioma } from "./i18n";
import "./estilo.css";
import "./componentes.css";
// O CSS do aplicativo, escopado sob `.vitrine` — ver scripts/importar-css.mjs.
import "./vitrine-app.css";

createRoot(document.getElementById("raiz")!).render(
  <StrictMode>
    <ProvedorIdioma>
      <App />
    </ProvedorIdioma>
  </StrictMode>
);
