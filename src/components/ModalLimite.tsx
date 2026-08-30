import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirEsperaLimite, ouvirFimDoLimite, ouvirLimite } from "../api";
import { useIdioma } from "../i18n";
import type { AvisoLimite, EsperaLimite } from "../types";

/** Quanto falta, em texto curto. */
function faltam(ate: number): string {
  const s = Math.max(0, ate - Math.floor(Date.now() / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return h > 0 ? `${h}h ${m}min` : `${m}min`;
}

function relogio(ts: number, idioma: string): string {
  return new Date(ts * 1000).toLocaleTimeString(idioma === "pt" ? "pt-BR" : "en-US", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** A cota do Claude Code acabou no meio da campanha.
 *
 *  Sem este aviso a campanha morreria com um erro genérico, e a pessoa — que
 *  saiu da frente do computador justamente porque a campanha demora — voltaria
 *  horas depois para encontrar tudo parado por um motivo que já tinha passado.
 *
 *  Duas saídas, e a diferença entre elas é quem espera: encerrar agora deixa os
 *  turnos já rodados gravados e devolve o controle; esperar deixa o próprio
 *  sistema dormir até a cota voltar e seguir sozinho. */
export default function ModalLimite() {
  const { d, f, idioma } = useIdioma();
  const [aviso, setAviso] = useState<AvisoLimite | null>(null);
  const [espera, setEspera] = useState<EsperaLimite | null>(null);
  const [agora, setAgora] = useState(Date.now());

  useEffect(() => {
    const us = [
      ouvirLimite((e) => {
        setEspera(null);
        setAviso(e);
      }),
      ouvirEsperaLimite((e) => {
        setAviso(null);
        setEspera(e);
      }),
      ouvirFimDoLimite(() => {
        setAviso(null);
        setEspera(null);
      }),
    ];
    return () => {
      us.forEach((p) => void p.then((u) => u()));
    };
  }, []);

  // O relógio só corre enquanto há espera na tela: um timer permanente
  // redesenharia a janela a cada segundo pelo resto da sessão.
  useEffect(() => {
    if (!espera) return;
    const t = setInterval(() => setAgora(Date.now()), 30_000);
    return () => clearInterval(t);
  }, [espera]);

  const responder = async (esperar: boolean) => {
    setAviso(null);
    await api.responderLimite(esperar).catch(() => {});
  };

  const aberto = aviso ?? espera;
  if (!aberto) return null;

  return (
    <AnimatePresence>
      <motion.div
        className="veu"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
      >
        <motion.div
          className="modal"
          role="dialog"
          aria-modal="true"
          initial={{ opacity: 0, y: 12, scale: 0.98 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ type: "spring", stiffness: 300, damping: 26 }}
        >
          {aviso ? (
            <>
              <span className="tag" data-tone="warn">
                <span className="tag__dot" />
                {d.limite.pilula}
              </span>
              <h2>{d.limite.titulo}</h2>
              <p>
                {aviso.volta_em
                  ? f(d.limite.textoCom, {
                      hora: relogio(aviso.volta_em, idioma),
                      falta: faltam(aviso.volta_em),
                    })
                  : d.limite.textoSem}
              </p>

              {/* A saída crua do CLI, para conferir. Um aviso que só afirma
                  "acabou a cota" sem mostrar de onde tirou isso não dá para
                  checar — e limite é o tipo de coisa que a pessoa quer ver
                  com os próprios olhos antes de aceitar esperar horas. */}
              <details className="porque">
                <summary>{d.limite.oQueOClaudeDisse}</summary>
                <pre className="raw porque__corpo">{aviso.evidencia}</pre>
              </details>

              <div className="modal__acoes">
                <button className="btn" onClick={() => void responder(false)}>
                  {d.limite.encerrar}
                </button>
                {aviso.volta_em && (
                  <button className="btn btn--key" onClick={() => void responder(true)}>
                    {f(d.limite.esperar, { falta: faltam(aviso.volta_em) })}
                  </button>
                )}
              </div>
              <p className="hint">
                {aviso.volta_em ? d.limite.nota : d.limite.notaSem}
              </p>
            </>
          ) : (
            espera && (
              <>
                <span className="tag" data-tone="live">
                  <span className="tag__dot" />
                  {d.limite.esperando}
                </span>
                <h2>{f(d.limite.voltaAs, { hora: relogio(espera.volta_em, idioma) })}</h2>
                <p key={agora}>
                  {f(d.limite.textoEsperando, { falta: faltam(espera.volta_em) })}
                </p>
                <div className="modal__acoes">
                  <button className="btn" onClick={() => void responder(false)}>
                    {d.limite.encerrarAgora}
                  </button>
                </div>
              </>
            )
          )}
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
