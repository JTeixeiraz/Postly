import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirFalha } from "../api";
import { useIdioma } from "../i18n";
import type { Falha } from "../types";
import { IconAlert, IconOpen } from "./Icons";
import { useOuvinte } from "../ouvir";

/** O que a pessoa encontra quando volta e a campanha morreu.
 *
 *  Uma campanha leva dezenas de minutos, e o produto existe justamente para
 *  que ninguém fique olhando: uma nota no rodapé é um erro que não é visto.
 *  Aqui a notificação do sistema chama de volta, e o modal diz o que
 *  aconteceu, o que fazer e onde está o rastro. */
export default function ModalFalha() {
  const { d } = useIdioma();
  const [falha, setFalha] = useState<Falha | null>(null);

  useOuvinte(() => ouvirFalha(setFalha), []);

  // Escape fecha: o modal informa, não retém. Reter o que já falhou só
  // somaria uma segunda frustração à primeira.
  useEffect(() => {
    if (!falha) return;
    const aoTeclar = (e: KeyboardEvent) => e.key === "Escape" && setFalha(null);
    window.addEventListener("keydown", aoTeclar);
    return () => window.removeEventListener("keydown", aoTeclar);
  }, [falha]);

  return (
    <AnimatePresence>
      {falha && (
        <motion.div
          className="veu"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={() => setFalha(null)}
        >
          <motion.div
            className="modal modal--falha"
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="falha-titulo"
            initial={{ opacity: 0, y: 18, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 10, scale: 0.99 }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal__topo">
              <span className="modal__selo" data-tone="alert">
                <IconAlert size={15} />
              </span>
              <div>
                <h2 id="falha-titulo">{d.falha.titulo}</h2>
                <span className="hint">{falha.etapa}</span>
              </div>
            </div>

            {/* A sugestão vem antes do detalhe: quem está travado quer saber o
                que fazer, não o que a biblioteca escreveu. */}
            {falha.sugestao && (
              <div className="note" data-tone="warn">
                <strong>{d.falha.oQueFazer}</strong>
                <span>{falha.sugestao}</span>
              </div>
            )}

            <div className="field">
              <span>{d.falha.mensagem}</span>
              <pre className="raw modal__detalhe">{falha.detalhe}</pre>
            </div>

            <span className="hint">{d.falha.rastro}</span>

            <div className="row">
              {falha.pasta && (
                <button className="btn btn--sm" onClick={() => api.abrirNoSistema(falha.pasta!)}>
                  <IconOpen size={13} />
                  {d.falha.abrirPasta}
                </button>
              )}
              <button
                className="btn btn--sm"
                onClick={() => void navigator.clipboard?.writeText(falha.detalhe)}
              >
                {d.falha.copiar}
              </button>
              <span className="push" />
              <button className="btn btn--key btn--sm" onClick={() => setFalha(null)}>
                {d.common.close}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
