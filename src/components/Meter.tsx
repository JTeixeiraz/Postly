import { formatarBytes, useIdioma } from "../i18n";
import type { RamSnapshot } from "../types";

/** Medidor de memória: quanto está em uso, e onde fica o teto de um modelo.
 *
 *  A marca do teto é o que dá sentido ao resto: sem ela, "16 GB em uso" não diz
 *  se ainda cabe um gerente ou não. */
export default function Meter({ ram }: { ram: RamSnapshot }) {
  const { idioma } = useIdioma();
  const uso = ram.total_bytes ? (ram.used_bytes / ram.total_bytes) * 100 : 0;
  const teto = ram.total_bytes ? (ram.max_budget_bytes / ram.total_bytes) * 100 : 0;

  return (
    <div
      className="meter"
      role="meter"
      aria-valuenow={Math.round(uso)}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={`${formatarBytes(ram.used_bytes, idioma)} / ${formatarBytes(ram.total_bytes, idioma)}`}
    >
      <div className="meter__fill" data-p={ram.pressure} style={{ width: `${uso}%` }} />
      <div className="meter__cap" style={{ left: `${teto}%` }} />
    </div>
  );
}
