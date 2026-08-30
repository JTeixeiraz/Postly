import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import Janela, { type Aba } from "../vitrine/Janela";
import { TelaCampanha, TelaCerebro, TelaModelos } from "../vitrine/Telas";
import { useIdioma } from "../i18n";

/** O aplicativo rodando dentro da página.
 *
 *  Não são capturas: é o mesmo React, o mesmo CSS e — no Cérebro — o mesmo
 *  canvas com a física de forças que roda no produto. Um print mostraria a
 *  mesma tela parada, e o que este produto faz é justamente se mover. */
export default function Vitrine() {
  const { d } = useIdioma();
  const [aba, setAba] = useState<Aba>("modelos");
  // O turno em que a trilha está, para a campanha não ficar num quadro morto.
  const [turno, setTurno] = useState(1);

  return (
    <div className="vitrine-bloco">
      <Janela aba={aba} onAba={setAba}>
        <AnimatePresence mode="wait">
          <motion.div
            key={aba}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
          >
            {aba === "modelos" && <TelaModelos />}
            {aba === "campanha" && <TelaCampanha ate={turno} />}
            {aba === "cerebro" && <TelaCerebro />}
          </motion.div>
        </AnimatePresence>
      </Janela>

      <div className="vitrine-bloco__pe">
        <p className="vitrine-bloco__legenda">{d.telas.legendas[aba]}</p>
        {aba === "campanha" && (
          <button className="acao acao--fantasma" onClick={() => setTurno((t) => (t + 1) % 5)}>
            {d.telas.avancar}
          </button>
        )}
      </div>
    </div>
  );
}
