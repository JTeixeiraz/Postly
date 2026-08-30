import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import Fronteira from "./components/Fronteira";
import { ProvedorIdioma } from "./i18n";
import "./styles.css";
import "./components.css";
import "./config.css";
// Por ultimo: as media queries precisam vencer as regras de base.
import "./adapta.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Fronteira>
      <ProvedorIdioma>
        <App />
      </ProvedorIdioma>
    </Fronteira>
  </React.StrictMode>
);
