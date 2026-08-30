import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { useIdioma } from "../i18n";

type Modo = "escuro" | "claro";
const CHAVE = "postly:tema";

/** Troca entre o tema escuro e o claro.
 *
 *  O padrão segue o sistema operacional: quem já escolheu claro no computador
 *  não deveria precisar escolher de novo aqui. A escolha feita neste botão
 *  passa a valer sobre a do sistema, porque é a mais específica — e fica
 *  guardada, senão toda abertura desfaria a decisão. */
export default function Tema() {
  const { d } = useIdioma();
  const [modo, setModo] = useState<Modo>(() => {
    const salvo = localStorage.getItem(CHAVE);
    if (salvo === "claro" || salvo === "escuro") return salvo;
    return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "claro" : "escuro";
  });

  useEffect(() => {
    // O atributo mora no <html>: os tokens são redefinidos em `:root`, e um
    // seletor mais fundo não alcançaria o `body`.
    document.documentElement.dataset.tema = modo;
    localStorage.setItem(CHAVE, modo);
  }, [modo]);

  const claro = modo === "claro";

  return (
    <button
      className="tema"
      onClick={() => setModo(claro ? "escuro" : "claro")}
      aria-pressed={claro}
      title={claro ? d.tema.paraEscuro : d.tema.paraClaro}
      aria-label={claro ? d.tema.paraEscuro : d.tema.paraClaro}
    >
      {/* Um trilho com a marca deslizando, não dois ícones que piscam: o
          movimento diz que é um interruptor de dois estados. */}
      <motion.span
        className="tema__marca"
        layout
        transition={{ type: "spring", stiffness: 500, damping: 34 }}
        style={{ marginLeft: claro ? "auto" : 0 }}
      />
      <span className="tema__glifos" aria-hidden>
        <Lua />
        <Sol />
      </span>
    </button>
  );
}

const Lua = () => (
  <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
    <path
      d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinejoin="round"
    />
  </svg>
);

const Sol = () => (
  <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
    <circle cx="12" cy="12" r="4" stroke="currentColor" strokeWidth="1.8" />
    <path
      d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
    />
  </svg>
);
