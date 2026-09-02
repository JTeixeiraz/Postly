import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirNarracao } from "../api";
import { useIdioma } from "../i18n";
import type { PedidoNarracao } from "../types";
import { useOuvinte } from "../ouvir";

/** A pergunta sobre narração, quando a pasta de voz está vazia.
 *
 *  Isto para o vídeo de verdade: o Rust dorme num canal até a resposta chegar.
 *  Por isso o modal não fecha sem responder, e por isso a notificação do
 *  sistema sai junto do evento — a mesma lição do turno de movimento.
 *
 *  A pergunta chega DEPOIS de o gerente decidir a linha e ANTES de o motion
 *  designer montar a primeira cena. Antes seria sem contexto; depois seria
 *  tarde, porque as cenas já teriam sido medidas para texto na tela. */
export default function ModalNarracao() {
  const { d } = useIdioma();
  const [pedido, setPedido] = useState<PedidoNarracao | null>(null);
  const [enviando, setEnviando] = useState(false);

  useOuvinte(() => ouvirNarracao(setPedido), []);

  const responder = async (quer: boolean) => {
    setEnviando(true);
    try {
      await api.responderNarracao(quer ? "quero_roteiro" : "sem_voz");
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
          aria-label={d.narracao.title}
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
              {d.narracao.waiting}
            </span>

            <h2>{d.narracao.title}</h2>
            <p className="hint">{d.narracao.why}</p>

            <div className="modal__campo">
              <span className="read__k">{d.narracao.line}</span>
              <p>{pedido.linha}</p>
            </div>

            <div className="row">
              <button
                className="btn btn--key"
                disabled={enviando}
                onClick={() => void responder(true)}
              >
                {d.narracao.accept}
              </button>
              <button className="btn" disabled={enviando} onClick={() => void responder(false)}>
                {d.narracao.decline}
              </button>
            </div>

            {/* O que cada botão faz, dito antes do clique. "Quero narração" não
                gera voz nenhuma: entrega um roteiro para a pessoa gravar e para
                o vídeo até o arquivo chegar. Descobrir isso depois de clicar
                seria a interface prometendo o que não acontece. */}
            <p className="hint">{d.narracao.acceptWhat}</p>
            <p className="hint">{d.narracao.declineWhat}</p>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
