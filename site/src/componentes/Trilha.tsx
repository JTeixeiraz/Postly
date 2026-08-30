import { motion, useReducedMotion } from "motion/react";

export interface Posta {
  cargo: string;
  modelo: string;
  nota: string;
}

/** A trilha de revezamento, animada.
 *
 *  É a única coisa no site que se move sozinha ao carregar, e ela ganha esse
 *  direito por ser o produto: um despacho atravessando quatro cargos, trocando
 *  de modelo a cada perna. A animação é a explicação.
 *
 *  Cada posta acende quando o despacho chega nela e a linha avança junto. Sem
 *  motion, todas aparecem acesas de uma vez: o estado final é o correto, então
 *  a página nunca fica pela metade. */
export default function Trilha({ postas }: { postas: Posta[] }) {
  const parado = useReducedMotion();
  const passo = 0.55;

  return (
    <div className="trilha" role="img" aria-label={`Percurso: ${postas.map((p) => p.cargo).join(", então ")}`}>
      <div className="trilha__fio" aria-hidden />
      <motion.div
        className="trilha__aceso"
        aria-hidden
        initial={{ scaleX: parado ? 1 : 0 }}
        animate={{ scaleX: 1 }}
        transition={{ duration: parado ? 0 : postas.length * passo, ease: "easeInOut" }}
      />

      {postas.map((p, i) => (
        <motion.div
          className="posta-t"
          key={p.cargo}
          initial={{ opacity: parado ? 1 : 0.32 }}
          animate={{ opacity: 1 }}
          transition={{ delay: parado ? 0 : i * passo, duration: 0.4, ease: [0.16, 1, 0.3, 1] }}
        >
          <motion.span
            className="posta-t__marca"
            aria-hidden
            initial={{ scale: parado ? 1 : 0.55, backgroundColor: "rgba(0,0,0,0)" }}
            animate={{ scale: 1, backgroundColor: "var(--act)" }}
            transition={{ delay: parado ? 0 : i * passo, duration: 0.42, ease: [0.16, 1, 0.3, 1] }}
          />
          <span className="posta-t__cargo">{p.cargo}</span>
          <span className="posta-t__modelo mono">{p.modelo}</span>
          <span className="posta-t__nota">{p.nota}</span>
        </motion.div>
      ))}
    </div>
  );
}
