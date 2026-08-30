import { useEffect, useState } from "react";
import { api } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { PlanoOtimizacao, RelatorioOtimizacao } from "../types";
import { IconBroom, IconCheck } from "../components/Icons";

/** Liberação de memória.
 *
 *  Duas frentes com custos diferentes: descarregar modelo residente devolve RAM
 *  na hora e não perde nada; apagar cache devolve disco e obriga o programa dono
 *  a reconstruir. Por isso cada alvo é aprovado individualmente, e o passo que
 *  pede senha fica separado dos outros. */
export default function Otimizar({
  onFechar,
  onLimpou,
}: {
  onFechar: () => void;
  onLimpou: () => void | Promise<void>;
}) {
  const { d, f, idioma } = useIdioma();
  const [plano, setPlano] = useState<PlanoOtimizacao | null>(null);
  const [escolhidos, setEscolhidos] = useState<string[]>([]);
  const [elevar, setElevar] = useState(false);
  const [relatorio, setRelatorio] = useState<RelatorioOtimizacao | null>(null);
  const [rodando, setRodando] = useState(false);

  useEffect(() => {
    void api.planoOtimizacao().then((p) => {
      setPlano(p);
      setEscolhidos(p.targets.filter((t) => t.safe).map((t) => t.path));
    });
  }, []);

  const executar = async () => {
    setRodando(true);
    try {
      setRelatorio(await api.otimizar(escolhidos, elevar));
      await onLimpou();
    } finally {
      setRodando(false);
    }
  };

  return (
    <section className="card">
      <div className="card__topo">
        <h2>{d.boot.optimize}</h2>
        <button className="btn btn--quiet btn--sm push" onClick={onFechar}>
          {d.common.close}
        </button>
      </div>
      <p className="hint" style={{ marginBottom: 20 }}>
        {d.boot.optimizeLead}
      </p>

      {!plano && <div className="skeleton" style={{ height: 90 }} />}

      {plano && (
        <div className="card">
          {plano.targets.length === 0 && <p className="hint">—</p>}

          {plano.targets.map((alvo) => (
            <label className="choice" key={alvo.path} data-on={escolhidos.includes(alvo.path)}>
              <input
                type="checkbox"
                checked={escolhidos.includes(alvo.path)}
                onChange={(e) =>
                  setEscolhidos((s) =>
                    e.target.checked ? [...s, alvo.path] : s.filter((p) => p !== alvo.path)
                  )
                }
              />
              <div>
                <div className="row row--tight">
                  <span className="choice__title">{alvo.label}</span>
                  <span className="push modelo__peso">{formatarBytes(alvo.bytes, idioma)}</span>
                </div>
                <div className="hint mono" style={{ overflowWrap: "anywhere" }}>
                  {alvo.path}
                </div>
              </div>
            </label>
          ))}

          {plano.drop_caches && (
            <label className="choice" data-on={elevar}>
              <input type="checkbox" checked={elevar} onChange={(e) => setElevar(e.target.checked)} />
              <div>
                <span className="choice__title">{plano.drop_caches.label}</span>
                <div className="hint">{d.boot.optimizeElevate}</div>
              </div>
            </label>
          )}

          <div className="row">
            <span className="hint">
              {formatarBytes(plano.reclaimable_bytes, idioma)}
            </span>
            <button className="btn btn--key push" onClick={executar} disabled={rodando}>
              <IconBroom size={16} />
              {rodando ? d.boot.probing : d.boot.optimizeRun}
            </button>
          </div>
        </div>
      )}

      {relatorio && (
        <div className="note" data-tone="signal" style={{ marginTop: 16 }}>
          <div className="row row--tight">
            <IconCheck size={15} />
            <strong>
              {f(d.boot.optimizeDone, {
                ram: formatarBytes(
                  Math.max(relatorio.after.available_bytes - relatorio.before.available_bytes, 0),
                  idioma
                ),
                disk: formatarBytes(relatorio.freed_disk_bytes, idioma),
              })}
            </strong>
          </div>
          {relatorio.actions.map((a, i) => (
            <div className="hint" key={i}>
              {a}
            </div>
          ))}
          {relatorio.failures.map((x, i) => (
            <div className="hint" key={i} style={{ color: "var(--alert)" }}>
              {x}
            </div>
          ))}
        </div>
      )}

      {plano && plano.hogs.length > 0 && (
        <div style={{ marginTop: 24 }}>
          <h3 style={{ marginBottom: 12 }}>{d.boot.hogs}</h3>
          <div className="table-wrap">
            <table className="data">
              <tbody>
                {plano.hogs.map((h) => (
                  <tr key={h.pid}>
                    <td>{h.name}</td>
                    <td className="n dim">{h.pid}</td>
                    <td className="n">{formatarBytes(h.memory_bytes, idioma)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="hint" style={{ marginTop: 10 }}>
            {d.boot.hogsNote}
          </p>
        </div>
      )}
    </section>
  );
}
