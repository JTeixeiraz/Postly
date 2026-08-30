import { convertFileSrc } from "@tauri-apps/api/core";
import { motion } from "motion/react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { RelatorioCampanha } from "../types";
import { IconAlert, IconCheck, IconOpen } from "./Icons";

/** O que a campanha produziu: veredito, avisos e as peças. */
export default function Resultado({ relatorio }: { relatorio: RelatorioCampanha }) {
  const { d, f } = useIdioma();

  return (
    <motion.section
      className="card card--light"
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ type: "spring", stiffness: 220, damping: 28 }}
    >
      <div className="card__topo">
        <h2>{relatorio.run_id}</h2>
        <span className="tag" data-tone={relatorio.aprovado ? "ok" : "alert"}>
          <span className="tag__dot" />
          {f(relatorio.aprovado ? d.campaign.approved : d.campaign.rejected, {
            n: relatorio.rodadas,
          })}
        </span>
        <span className="push" />
        <button className="btn btn--quiet btn--sm" onClick={() => api.abrirNoSistema(relatorio.run_dir)}>
          <IconOpen size={14} />
          {d.campaign.openRun}
        </button>
        <button className="btn btn--quiet btn--sm" onClick={() => api.fecharNavegador()}>
          {d.campaign.closeBrowser}
        </button>
      </div>

      {relatorio.parecer_auditor && (
        <div className="note" style={{ marginBottom: 16 }}>
          <strong>{d.campaign.verdict}</strong>
          <span>{relatorio.parecer_auditor}</span>
        </div>
      )}

      {relatorio.avisos.length > 0 && (
        <div className="note" data-tone="alert" style={{ marginBottom: 20 }}>
          <div className="row row--tight">
            <IconAlert size={14} />
            <strong>{d.campaign.warnings}</strong>
          </div>
          {relatorio.avisos.map((a, i) => (
            <span key={i} className="hint">
              {a}
            </span>
          ))}
        </div>
      )}

      <div className="auto-grid auto-grid--wide cascata">
        {relatorio.pecas.map((peca, i) => (
          <article className="peca" key={i}>
            {peca.imagem && (
              <img
                className="peca__arte"
                src={convertFileSrc(peca.imagem.path)}
                alt={peca.conceito || peca.rede}
                loading="lazy"
              />
            )}
            <div className="peca__corpo">
              <div className="row row--tight">
                <strong style={{ color: "var(--ink)" }}>{peca.rede}</strong>
                <span className="tag" data-tone={peca.publicado ? "ok" : "alert"}>
                  <span className="tag__dot" />
                  {peca.publicado ? d.campaign.published : d.campaign.notPublished}
                </span>
              </div>

              {peca.conceito && <p className="hint">{peca.conceito}</p>}
              <p className="peca__legenda">{peca.legenda}</p>
              {peca.hashtags.length > 0 && <div className="peca__tags">{peca.hashtags.join(" ")}</div>}

              {peca.chamada_para_acao && (
                <p className="hint">
                  <IconCheck size={13} /> {d.campaign.cta}: {peca.chamada_para_acao}
                </p>
              )}
              {peca.detalhe_publicacao && <p className="hint">{peca.detalhe_publicacao}</p>}

              {/* O roteiro fica recolhido: e uma tabela de cenas, longa, e quem
                  vem conferir a peca quer ver a peca primeiro. */}
              {peca.roteiro_motion && (
                <details className="porque">
                  <summary>{d.motion.scriptTitle}</summary>
                  <pre className="raw porque__corpo">{peca.roteiro_motion}</pre>
                </details>
              )}

              <div className="row row--tight">
                {peca.imagem && (
                  <button
                    className="btn btn--quiet btn--sm"
                    onClick={() => api.abrirNoSistema(peca.imagem!.path)}
                  >
                    {d.campaign.openImage}
                  </button>
                )}
                {peca.screenshot && (
                  <button
                    className="btn btn--quiet btn--sm"
                    onClick={() => api.abrirNoSistema(peca.screenshot!)}
                  >
                    {d.campaign.openShot}
                  </button>
                )}
              </div>
            </div>
          </article>
        ))}
      </div>
    </motion.section>
  );
}
