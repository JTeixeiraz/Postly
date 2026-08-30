import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useIdioma } from "../i18n";
import { IconHelp } from "./Icons";
import Selecao from "./Selecao";

const TOPICOS = [
  "geral",
  "revezamento",
  "cargos",
  "cerebro",
  "referencias",
  "modo",
  "avancado",
] as const;

/** Explicacao do sistema, por assunto.
 *
 *  Fica atras de um botao, e nao espalhada pelas telas, porque quem abre o app
 *  pela segunda vez nao precisa reler nada. Um seletor em vez de um passo a
 *  passo forcado: a pessoa vem aqui com uma duvida especifica, e obrigar a
 *  atravessar sete telas para chegar nela seria pior do que nao ter. */
export default function Tour() {
  const { d } = useIdioma();
  const [aberto, setAberto] = useState(false);
  const [topico, setTopico] = useState<(typeof TOPICOS)[number]>("geral");
  const item = d.tour.items[topico];

  return (
    <>
      <button className="btn btn--quiet btn--sm tour__abre" onClick={() => setAberto(true)}>
        <IconHelp size={15} />
        <span>{d.tour.open}</span>
      </button>

      <AnimatePresence>
        {aberto && (
          <motion.div
            className="tour__fundo"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.16 }}
            onClick={() => setAberto(false)}
          >
            <motion.div
              className="tour card"
              initial={{ opacity: 0, y: 14, scale: 0.98 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 8, scale: 0.99 }}
              transition={{ type: "spring", stiffness: 260, damping: 26 }}
              onClick={(e) => e.stopPropagation()}
              role="dialog"
              aria-label={d.tour.title}
            >
              <div className="card__topo">
                <h2>{d.tour.title}</h2>
                <button className="btn btn--quiet btn--sm push" onClick={() => setAberto(false)}>
                  {d.common.close}
                </button>
              </div>

              <label className="field">
                <span>{d.tour.pick}</span>
                <Selecao
                  valor={topico}
                  onEscolher={(v) => setTopico(v as (typeof TOPICOS)[number])}
                  opcoes={TOPICOS.map((id) => ({ valor: id, rotulo: d.tour.items[id].t }))}
                />
              </label>

              {/* A troca de assunto anima: sem isso o texto salta e a pessoa
                  perde a referencia de que algo mudou. */}
              <AnimatePresence mode="wait">
                <motion.div
                  key={topico}
                  className="card__corpo"
                  initial={{ opacity: 0, y: 6 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -4 }}
                  transition={{ duration: 0.18 }}
                >
                  <h3>{item.t}</h3>
                  <p style={{ marginTop: 8 }}>{item.d}</p>
                </motion.div>
              </AnimatePresence>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
