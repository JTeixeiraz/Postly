import { useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useIdioma } from "../i18n";

/** O vídeo de apresentação, carregado sob demanda.
 *
 *  São 4 MB, com narração e trilha. Numa landing page isso é a diferença entre a página abrir e a
 *  página esperar, então o que carrega junto é o pôster de 20 KB e o `<video>`
 *  só nasce quando alguém aperta o play. `preload="none"` não bastaria: o
 *  elemento ainda assim resolve o arquivo em alguns navegadores.
 *
 *  Depois de iniciado ele vira um player nativo — controles próprios seriam
 *  mais um sistema para manter, e o do navegador já é acessível e conhecido. */
export default function Apresentacao() {
  const { d } = useIdioma();
  const [tocando, setTocando] = useState(false);
  const ref = useRef<HTMLVideoElement>(null);

  return (
    <div className="video">
      <div className="video__moldura">
        <AnimatePresence mode="wait">
          {tocando ? (
            <motion.video
              key="video"
              ref={ref}
              className="video__quadro"
              src="postly.mp4"
              poster="poster.jpg"
              controls
              autoPlay
              playsInline
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
            />
          ) : (
            <motion.button
              key="capa"
              className="video__capa"
              onClick={() => setTocando(true)}
              aria-label={d.video.assistir}
              initial={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.25 }}
            >
              <img className="video__quadro" src="poster.jpg" alt="" />
              <span className="video__play">
                <svg width="26" height="26" viewBox="0 0 24 24" aria-hidden>
                  <path d="M8 5.5v13l11-6.5z" fill="currentColor" />
                </svg>
              </span>
              <span className="video__duracao">{d.video.duracao}</span>
            </motion.button>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
