import { useEffect, useState } from "react";
import { motion } from "motion/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "../api";
import { formatarExecucao, useIdioma } from "../i18n";
import type { CampanhaSalva, PecaSalva, ResultadoCampanha } from "../types";
import { IconArchive, IconCheck, IconOpen } from "../components/Icons";

const ROTULO_REDE: Record<string, string> = {
  instagram: "Instagram",
  facebook: "Facebook",
  tiktok: "TikTok",
  linkedin: "LinkedIn",
  x: "X",
};

/** O arquivo do que os agentes produziram.
 *
 *  Antes esta tela mostrava um despejo de markdown: a transcricao existia, mas
 *  a PECA — arte, legenda, hashtags — so vivia na memoria da janela e sumia ao
 *  fechar o app. Agora o resultado e gravado em disco junto da conversa, e o
 *  que se ve aqui e o trabalho, com a transcricao atras de um clique para quem
 *  quiser auditar como se chegou nele. */
export default function Historico() {
  const { d, f, idioma } = useIdioma();
  const [campanhas, setCampanhas] = useState<CampanhaSalva[] | null>(null);
  const [aberta, setAberta] = useState<string | null>(null);
  const [resultado, setResultado] = useState<ResultadoCampanha | null>(null);
  const [bruto, setBruto] = useState<string | null>(null);

  useEffect(() => {
    void api.listarCampanhas().then(setCampanhas);
  }, []);

  const abrir = async (c: CampanhaSalva) => {
    if (aberta === c.id) {
      setAberta(null);
      return;
    }
    setAberta(c.id);
    setBruto(null);
    setResultado(await api.pecasDaCampanha(c.dir).catch(() => null));
  };

  const verBruto = async (c: CampanhaSalva) => {
    if (bruto !== null) {
      setBruto(null);
      return;
    }
    setBruto(await api.lerMarkdown(c.index).catch((e) => String(e)));
  };

  return (
    <>
      <header className="page__head">
        <h1>{d.history.title}</h1>
        <p>{d.history.lead}</p>
      </header>

      {!campanhas && <div className="skeleton" style={{ height: 120 }} />}

      {campanhas?.length === 0 && (
        <div className="empty">
          <IconArchive size={26} />
          <h3>{d.history.empty}</h3>
          <p className="hint">{d.history.emptyHint}</p>
        </div>
      )}

      <div className="cascata">
        {campanhas?.map((c) => (
          <section className="card" key={c.id}>
            <div className="card__topo">
              {/* O objetivo e como a pessoa reconhece a campanha; a data e o
                  segundo criterio; o id fica para casar com a pasta no disco. */}
              <h2 className="run__titulo">
                {c.objetivo || (formatarExecucao(c.id, idioma) ?? c.id)}
              </h2>
              {c.pecas > 0 && (
                <span className="tag" data-tone={c.publicadas > 0 ? "ok" : undefined}>
                  <span className="tag__dot" />
                  {c.simulado
                    ? d.history.simulated
                    : f(d.history.publishedOf, { n: c.publicadas, total: c.pecas })}
                </span>
              )}
              <span className="push" />
              <button className="btn btn--sm" onClick={() => abrir(c)}>
                {aberta === c.id ? d.common.close : d.history.read}
              </button>
              <button
                className="btn btn--quiet btn--sm"
                onClick={() => api.abrirNoSistema(c.dir)}
                title={d.history.openFolder}
              >
                <IconOpen size={14} />
              </button>
            </div>

            <div className="run__meta">
              <span>{formatarExecucao(c.id, idioma) ?? c.id}</span>
              {c.redes.length > 0 && (
                <span>{c.redes.map((r) => ROTULO_REDE[r] ?? r).join(" · ")}</span>
              )}
              <span>
                {c.turns === 1 ? d.history.turnsOne : f(d.history.turnsMany, { n: c.turns })}
              </span>
              <span className="mono">{c.id}</span>
            </div>

            {aberta === c.id && (
              <motion.div
                initial={{ opacity: 0, y: -6 }}
                animate={{ opacity: 1, y: 0 }}
                className="stack"
              >
                {!resultado ? (
                  <p className="hint">{d.history.noResult}</p>
                ) : (
                  <>
                    {resultado.pecas.length === 0 && (
                      <p className="hint">{d.history.noPieces}</p>
                    )}
                    <div className="galeria">
                      {resultado.pecas.map((p, i) => (
                        <Peca key={`${p.rede}-${i}`} p={p} d={d} />
                      ))}
                    </div>

                    {resultado.parecer_auditor && (
                      <details className="porque">
                        <summary>{d.history.auditorOpinion}</summary>
                        <p className="porque__corpo">{resultado.parecer_auditor}</p>
                      </details>
                    )}

                    {resultado.avisos.length > 0 && (
                      <div className="note" data-tone="warn">
                        {resultado.avisos.map((a, i) => (
                          <span key={i}>{a}</span>
                        ))}
                      </div>
                    )}
                  </>
                )}

                {/* A conversa inteira continua acessivel, atras de um clique:
                    ela e para auditar o caminho, nao para ler todo dia. */}
                <button
                  className="btn btn--quiet btn--sm"
                  style={{ justifySelf: "start" }}
                  onClick={() => void verBruto(c)}
                >
                  {bruto === null ? d.history.showTranscript : d.common.close}
                </button>
                {bruto !== null && <pre className="raw">{bruto}</pre>}
              </motion.div>
            )}
          </section>
        ))}
      </div>
    </>
  );
}

function Peca({ p, d }: { p: PecaSalva; d: any }) {
  return (
    <article className="peca">
      {p.imagem && (
        <img className="peca__arte" src={convertFileSrc(p.imagem.path)} alt={p.conceito || p.rede} />
      )}
      <div className="peca__corpo">
        <div className="row row--tight">
          <strong style={{ color: "var(--ink)" }}>{ROTULO_REDE[p.rede] ?? p.rede}</strong>
          <span className="tag" data-tone={p.publicado ? "ok" : undefined}>
            <span className="tag__dot" />
            {p.publicado ? d.campaign.published : d.campaign.notPublished}
          </span>
          {p.roteiro_motion && (
            <span className="tag" data-tone="live">
              <span className="tag__dot" />
              {d.motion.tag}
            </span>
          )}
        </div>

        {p.conceito && <p className="hint">{p.conceito}</p>}
        <p className="peca__legenda">{p.legenda}</p>
        {p.hashtags.length > 0 && <div className="peca__tags">{p.hashtags.join(" ")}</div>}
        {p.chamada_para_acao && (
          <p className="hint">
            <IconCheck size={13} /> {d.campaign.cta}: {p.chamada_para_acao}
          </p>
        )}
        {p.detalhe_publicacao && <p className="hint">{p.detalhe_publicacao}</p>}

        {p.roteiro_motion && (
          <details className="porque">
            <summary>{d.motion.scriptTitle}</summary>
            <pre className="raw porque__corpo">{p.roteiro_motion}</pre>
          </details>
        )}
      </div>
    </article>
  );
}
