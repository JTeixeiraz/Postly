import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useIdioma } from "../i18n";
import { IconCheck } from "./Icons";
import type { Passo } from "./Guia";

/** Guia de primeiros passos, agora no topo.
 *
 *  Com a navegacao no header nao ha mais rodape de trilho onde ele morava. Vira
 *  um botao que mostra so quanto falta, e abre a lista sob demanda: o numero e
 *  o que a pessoa precisa de relance, a lista e o que ela abre quando decide
 *  agir. Some por inteiro quando termina, como antes. */
export default function GuiaTopo({ passos }: { passos: Passo[] }) {
  const { d, f } = useIdioma();
  const [aberto, setAberto] = useState(false);
  const caixa = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!aberto) return;
    const fora = (e: MouseEvent) => {
      if (!caixa.current?.contains(e.target as Node)) setAberto(false);
    };
    document.addEventListener("mousedown", fora);
    return () => document.removeEventListener("mousedown", fora);
  }, [aberto]);

  const faltam = passos.filter((p) => !p.feito);
  if (faltam.length === 0) return null;
  const proximo = faltam[0];

  return (
    <div className="guia-topo" ref={caixa}>
      <button className="guia-topo__gatilho" onClick={() => setAberto((a) => !a)} aria-expanded={aberto}>
        <span className="guia-topo__anel" aria-hidden>
          <span style={{ "--parte": (passos.length - faltam.length) / passos.length } as React.CSSProperties} />
        </span>
        <span>{d.guide.title}</span>
        <span className="guia-topo__conta">
          {faltam.length === 1
            ? f(d.guide.left, { n: faltam.length })
            : f(d.guide.leftMany, { n: faltam.length })}
        </span>
      </button>

      <AnimatePresence>
        {aberto && (
          <motion.div
            className="guia-topo__painel card"
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.99 }}
            transition={{ duration: 0.16, ease: [0.16, 1, 0.3, 1] }}
          >
            <ul className="guia__lista">
              {passos.map((p) => {
                const alvo = p.id === proximo.id;
                const conteudo = (
                  <>
                    <span className="guia__marca" aria-hidden>
                      {p.feito && <IconCheck size={11} />}
                    </span>
                    <span className="guia__rotulo">{p.rotulo}</span>
                  </>
                );
                return (
                  <li key={p.id} className="guia__item" data-feito={p.feito} data-alvo={alvo}>
                    {!p.feito && p.ir ? (
                      <button
                        type="button"
                        title={p.nota}
                        onClick={() => {
                          p.ir?.();
                          setAberto(false);
                        }}
                      >
                        {conteudo}
                      </button>
                    ) : (
                      <span title={p.nota}>{conteudo}</span>
                    )}
                  </li>
                );
              })}
            </ul>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
