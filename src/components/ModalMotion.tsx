import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirMotion } from "../api";
import { useIdioma } from "../i18n";
import type { PedidoMotion } from "../types";
import { useOuvinte } from "../ouvir";

const ROTULO_REDE: Record<string, string> = {
  instagram: "Instagram",
  facebook: "Facebook",
  tiktok: "TikTok",
  linkedin: "LinkedIn",
  x: "X",
};

/** A decisao sobre animar uma peca, quando o gerente pede movimento.
 *
 *  Isto para a campanha de verdade: o Rust dorme num canal ate a resposta
 *  chegar. Por isso o modal nao tem como fechar sem responder, e por isso a
 *  notificacao do sistema sai junto do evento — quem espera dezenas de minutos
 *  por um turno nao fica olhando a janela, e um modal que passa despercebido
 *  segura a campanha ate o tempo estourar. */
export default function ModalMotion() {
  const { d, f } = useIdioma();
  const [pedido, setPedido] = useState<PedidoMotion | null>(null);
  const [enviando, setEnviando] = useState(false);

  useOuvinte(() => ouvirMotion(setPedido), []);

  const responder = async (aceitar: boolean) => {
    setEnviando(true);
    try {
      await api.responderMotion(aceitar);
    } finally {
      setEnviando(false);
      setPedido(null);
    }
  };

  return (
    <AnimatePresence>
      {pedido && (
        <motion.div
          className="veu"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          role="dialog"
          aria-modal="true"
          aria-label={d.motion.title}
        >
          <motion.div
            className="modal"
            initial={{ opacity: 0, y: 14, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 8, scale: 0.98 }}
            transition={{ duration: 0.26, ease: [0.16, 1, 0.3, 1] }}
          >
            <span className="tag" data-tone="live">
              <span className="tag__dot" />
              {d.motion.waiting}
            </span>

            <h2>{d.motion.title}</h2>

            <div className="modal__campo">
              <span className="read__k">{d.motion.who}</span>
              <span>{f(d.motion.manager, { rede: ROTULO_REDE[pedido.rede] ?? pedido.rede })}</span>
            </div>

            <div className="modal__campo">
              <span className="read__k">{d.motion.reason}</span>
              <p>{pedido.motivo}</p>
            </div>

            <p className="hint">{d.motion.what}</p>
            <div className="note" data-tone="warn">
              <span>{d.motion.cost}</span>
            </div>

            <div className="row">
              <button
                className="btn btn--key"
                disabled={enviando}
                onClick={() => void responder(true)}
              >
                {d.motion.accept}
              </button>
              <button className="btn" disabled={enviando} onClick={() => void responder(false)}>
                {d.motion.decline}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
