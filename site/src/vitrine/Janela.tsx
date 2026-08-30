import { type ReactNode } from "react";
import { motion } from "motion/react";
import { IconChip, IconLayers, IconRelay, IconGraph } from "../app/Icons";

export type Aba = "modelos" | "campanha" | "cerebro";

const ABAS: { id: Aba; rotulo: string; icone: typeof IconChip }[] = [
  { id: "modelos", rotulo: "Modelos", icone: IconLayers },
  { id: "campanha", rotulo: "Campanha", icone: IconRelay },
  { id: "cerebro", rotulo: "Cérebro", icone: IconGraph },
];

/** A janela do aplicativo, viva.
 *
 *  O conteúdo aqui dentro é o código do app, com o CSS do app: trocar de aba
 *  troca de tela de verdade, e o grafo do Cérebro é o mesmo canvas que você
 *  arrasta no produto. Uma captura de tela mostraria a mesma coisa parada — e
 *  a coisa que o Postly faz é justamente se mover. */
export default function Janela({
  aba,
  onAba,
  children,
}: {
  aba: Aba;
  onAba: (a: Aba) => void;
  children: ReactNode;
}) {
  return (
    <div className="janela">
      <div className="vitrine">
        {/* `shell` é a moldura do próprio aplicativo: papel branco em volta do
            cartão escuro. Não é decoração do site — é como o Postly abre. */}
        <div className="shell janela__app">
          <header className="topo">
            <div className="marca">
              <MarcaP size={21} />
              <span className="marca__nome">postly</span>
            </div>

            <nav className="abas">
              {ABAS.map(({ id, rotulo, icone: Icone }) => (
                <button
                  key={id}
                  className="aba"
                  aria-current={aba === id ? "page" : undefined}
                  onClick={() => onAba(id)}
                >
                  {aba === id && (
                    <motion.span
                      className="aba__bolha"
                      layoutId="vitrine-aba"
                      transition={{ type: "spring", stiffness: 380, damping: 32 }}
                    />
                  )}
                  <Icone size={14} />
                  <span>{rotulo}</span>
                </button>
              ))}
            </nav>

            <div className="topo__acoes">
              <div className="ram-topo">
                <div className="meter" style={{ width: 52 }}>
                  <div className="meter__fill" style={{ width: "34%" }} />
                </div>
                <span className="ram-topo__txt num">21,5 GB livre</span>
              </div>
              <div className="lang">
                <button aria-pressed="true">PT</button>
                <button aria-pressed="false">EN</button>
              </div>
            </div>
          </header>

          <main className="main">
            <div className="page">{children}</div>
          </main>
        </div>
      </div>
    </div>
  );
}

function MarcaP({ size = 21 }: { size?: number }) {
  return (
    <svg width={Math.round(size * (14 / 21.4))} height={size} viewBox="9 5.6 14 21.4" aria-hidden>
      <path
        d="M11.6 24.5V9.4h5.1a3.9 3.9 0 0 1 0 7.8h-5.1"
        stroke="currentColor" strokeWidth="2.6" fill="none"
        strokeLinecap="round" strokeLinejoin="round"
      />
      <circle cx="11.6" cy="17.2" r="2.5" fill="currentColor" />
    </svg>
  );
}
