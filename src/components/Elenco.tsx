import { motion } from "motion/react";
import { formatarBytes, formatarNumero, useIdioma } from "../i18n";
import type { Vaga } from "../types";
import { IconDownload } from "./Icons";

/** O time que a máquina escala, na ordem em que trabalha.
 *
 *  Desenhado como trilha e não como grade de cartões: quatro cartões idênticos
 *  não dizem que existe uma ordem, e a ordem é justamente o que este produto
 *  faz. Também fecha a leitura de hardware com a consequência dela, para o
 *  diagnóstico não terminar em número solto. */
export default function Elenco({ vagas }: { vagas: Vaga[] }) {
  const { d, f, idioma } = useIdioma();
  const nivelLabel: Record<string, string> = {
    alto: d.models.tierHigh,
    medio: d.models.tierMid,
    baixo: d.models.tierLow,
  };

  return (
    <div className="elenco" style={{ "--postas": vagas.length } as React.CSSProperties}>
      {vagas.map((vaga, i) => (
        <motion.div
          className="vaga"
          key={vaga.cargo + i}
          data-vazia={!vaga.modelo_label}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: i * 0.07, type: "spring", stiffness: 240, damping: 28 }}
        >
          <span className="vaga__marca" />
          <span className="vaga__cargo">{vaga.cargo_label}</span>

          {vaga.modelo_label ? (
            <>
              <strong className="vaga__modelo">{vaga.modelo_label}</strong>
              <span className="vaga__tag">{vaga.modelo}</span>
              <span className="vaga__vel">
                {f(d.models.speed, { tps: formatarNumero(vaga.estimated_tps, idioma, 1) })}
                <span className="dim"> · {formatarBytes(vaga.footprint_bytes, idioma)}</span>
              </span>
              <span className="vaga__selos">
                <span className="tag">
                  <span className="tag__dot" />
                  {nivelLabel[vaga.nivel]}
                </span>
                {!vaga.instalado && (
                  <span className="tag">
                    <IconDownload size={11} />
                    {d.boot.crewNotDownloaded}
                  </span>
                )}
              </span>
            </>
          ) : (
            <strong className="vaga__modelo" style={{ color: "var(--alert)" }}>
              {d.boot.crewEmpty}
            </strong>
          )}

          {vaga.aviso && <span className="hint">{vaga.aviso}</span>}
        </motion.div>
      ))}
    </div>
  );
}
