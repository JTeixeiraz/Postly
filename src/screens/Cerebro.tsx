import { useCallback, useEffect, useState } from "react";
import { motion } from "motion/react";
import { api } from "../api";
import { formatarBytes, formatarNumero, useIdioma } from "../i18n";
import type { EstatisticasCerebro, GrafoCerebro, VistaNo } from "../types";
import Grafo from "../components/Grafo";
import { IconBroom, IconGraph } from "../components/Icons";
import Porque from "../components/Porque";

export default function Cerebro() {
  const { d, f, idioma } = useIdioma();
  const [grafo, setGrafo] = useState<GrafoCerebro | null>(null);
  const [stats, setStats] = useState<EstatisticasCerebro | null>(null);
  const [selecionado, setSelecionado] = useState<string | null>(null);
  const [vista, setVista] = useState<VistaNo | null>(null);
  const [limiar, setLimiar] = useState(0.35);
  const [topK, setTopK] = useState(6);
  const [novo, setNovo] = useState({ id: "", tipo: "conhecimento", contexto: "" });
  const [aviso, setAviso] = useState<string | null>(null);

  const carregar = useCallback(async () => {
    const [g, s] = await Promise.all([api.cerebroGrafo(), api.cerebroStats()]);
    setGrafo(g);
    setStats(s);
  }, []);

  useEffect(() => {
    void carregar();
  }, [carregar]);

  useEffect(() => {
    if (!selecionado) {
      setVista(null);
      return;
    }
    void api.cerebroNode(selecionado, limiar, topK).then(setVista);
  }, [selecionado, limiar, topK]);

  const gravar = async () => {
    if (novo.id.trim().length < 2) return;
    setStats(await api.cerebroEscreverNode(novo.id.trim(), novo.tipo, novo.contexto));
    setNovo({ id: "", tipo: "conhecimento", contexto: "" });
    await carregar();
  };

  const decair = async () => {
    const n = await api.cerebroDecair();
    setAviso(f(d.brain.decayDone, { n }));
    await carregar();
  };

  if (!grafo || !stats) return <div className="skeleton" style={{ height: 320 }} />;

  return (
    <>
      <header className="page__head">
        <h1>{d.brain.title}</h1>
        <p>{d.brain.lead}</p>
        <Porque>{d.brain.why}</Porque>
      </header>

      <section className="card">
        <div className="auto-grid">
          <div className="read">
            <span className="read__k">{d.brain.structure}</span>
            <span className="read__v">
              {stats.nodes} <small>{d.brain.nodes}</small>
            </span>
            <span className="read__note">{f(d.brain.edges, { n: stats.edges })}</span>
          </div>
          <div className="read">
            <span className="read__k">{d.brain.artifact}</span>
            <span className="read__v">{formatarBytes(stats.compressed_bytes, idioma)}</span>
            <span className="read__note">
              {f(d.brain.artifactNote, {
                raw: formatarBytes(stats.raw_bytes, idioma),
                pct: Math.round(stats.ratio * 100),
              })}
            </span>
          </div>
          <div className="stack stack--tight">
            <span className="read__k">{d.brain.upkeep}</span>
            <button className="btn btn--sm" onClick={decair} style={{ justifySelf: "start" }}>
              <IconBroom size={14} />
              {d.brain.decay}
            </button>
            <span className="read__note">{aviso ?? d.brain.decayWhy}</span>
          </div>
        </div>
        <p className="hint mono" style={{ marginTop: 16, overflowWrap: "anywhere" }}>
          {stats.path}
        </p>
      </section>

      <section className="card">
        <div className="grafo-wrap">
          <Grafo grafo={grafo} selecionado={selecionado} onSelecionar={setSelecionado} />
        </div>
        <span className="grafo-dica">{d.brain.graphHint} {d.brain.pan}</span>
      </section>

      <section className="card">
        <div className="card__topo">
          <h2>{d.brain.neighborhood}</h2>
        </div>

        {/* Os dois controles decidem o que o agente ve, e por isso dizem o que
            fazem: rotulo com nome de gente, valor visivel, e a consequencia
            escrita embaixo. "limiar" e "top-k" nao significam nada para quem
            abriu o app agora. */}
        <div className="ajustes">
          <label className="ajuste">
            <span className="ajuste__topo">
              <span className="ajuste__nome">{d.brain.threshold}</span>
              <span className="ajuste__valor num">{formatarNumero(limiar, idioma, 2)}</span>
            </span>
            <input
              type="range"
              min={0}
              max={0.95}
              step={0.05}
              value={limiar}
              onChange={(e) => setLimiar(Number(e.target.value))}
            />
            <span className="ajuste__nota">{d.brain.thresholdWhy}</span>
          </label>

          <label className="ajuste">
            <span className="ajuste__topo">
              <span className="ajuste__nome">{d.brain.topK}</span>
              <span className="ajuste__valor num">{topK}</span>
            </span>
            <input
              type="range"
              min={1}
              max={15}
              step={1}
              value={topK}
              onChange={(e) => setTopK(Number(e.target.value))}
            />
            <span className="ajuste__nota">{d.brain.topKWhy}</span>
          </label>
        </div>

        {!vista ? (
          <div className="empty">
            <IconGraph size={24} />
            <p className="hint">{d.brain.pickNode}</p>
          </div>
        ) : (
          <motion.div
            key={vista.node}
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
          >
            <div className="row row--tight">
              <strong className="mono" style={{ color: "var(--ink)" }}>
                {vista.node}
              </strong>
              <span className="tag">
                <span className="tag__dot" />
                {vista.type}
              </span>
            </div>
            <p className="hint" style={{ marginTop: 8 }}>
              {vista.context}
            </p>

            {vista.neighbors.length === 0 ? (
              <p className="hint" style={{ marginTop: 18 }}>
                {f(d.brain.noNeighbors, { t: formatarNumero(limiar, idioma, 2) })}
              </p>
            ) : (
              <div className="table-wrap" style={{ marginTop: 18 }}>
                <table className="data">
                  <thead>
                    <tr>
                      <th style={{ textAlign: "right" }}>{d.brain.weight}</th>
                      <th>{d.brain.relation}</th>
                      <th>{d.brain.node}</th>
                      <th>{d.brain.context}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {vista.neighbors.map((v) => (
                      <tr key={v.node + v.type}>
                        <td className="n" style={{ color: "var(--signal)" }}>
                          {formatarNumero(v.weight, idioma, 2)}
                        </td>
                        <td className="mono dim">{v.type}</td>
                        <td>
                          <button className="btn btn--quiet btn--sm mono" onClick={() => setSelecionado(v.node)}>
                            {v.node}
                          </button>
                        </td>
                        <td className="hint">{v.context}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </motion.div>
        )}
      </section>

      <section className="card">
        <div className="card__topo">
          <h2>{d.brain.write}</h2>
        </div>
        <p className="hint" style={{ marginBottom: 18 }}>
          {d.brain.writeLead}
        </p>
        <div className="auto-grid">
          <label className="field">
            <span>{d.brain.id}</span>
            <input
              type="text"
              value={novo.id}
              placeholder="plano_anual"
              onChange={(e) => setNovo((n) => ({ ...n, id: e.target.value }))}
            />
          </label>
          <label className="field">
            <span>{d.brain.type}</span>
            <input
              type="text"
              value={novo.tipo}
              onChange={(e) => setNovo((n) => ({ ...n, tipo: e.target.value }))}
            />
          </label>
        </div>
        <label className="field" style={{ marginTop: 14 }}>
          <span>{d.brain.context}</span>
          <textarea
            value={novo.contexto}
            onChange={(e) => setNovo((n) => ({ ...n, contexto: e.target.value }))}
          />
        </label>
        <div className="row" style={{ marginTop: 14 }}>
          <span className="push" />
          <button className="btn btn--key" onClick={gravar} disabled={novo.id.trim().length < 2}>
            {d.brain.writeNode}
          </button>
        </div>
      </section>
    </>
  );
}
