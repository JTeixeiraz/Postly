import { useState } from "react";
import { motion } from "motion/react";
import MarcaModelo from "../app/MarcaModelo";
import Grafo from "../app/Grafo";
import { IconCheck, IconDownload, IconSliders, IconTrash } from "../app/Icons";
import { GRAFO, MODELOS, POSTAS } from "./dados";
import type { ModeloCatalogo } from "./tipos";

const gb = (b: number) => `${(b / 1e9).toFixed(1).replace(".", ",")} GB`;
const n1 = (v: number) => v.toFixed(1).replace(".", ",");

/* ══ Modelos ═══════════════════════════════════════════════════════════════ */

export function TelaModelos() {
  const [familia, setFamilia] = useState("");
  const familias = [...new Set(MODELOS.map((m) => m.family))];
  const lista = MODELOS.filter((m) => !familia || m.family === familia);

  return (
    <>
      <header className="page__head">
        <h1>O que roda aqui</h1>
        <p>Você não escolhe. A cada cargo, sobe o modelo mais forte que couber.</p>
      </header>

      <section className="card">
        <div className="auto-grid">
          <Leitura k="teto por modelo" v="21,5 GB" nota="CPU apenas" />
          <Leitura k="cabe" v="7" small="/ 8" nota="1 fora do alcance deste hardware" />
          <Leitura k="baixado" v="2" nota="23,9 GB" />
        </div>
      </section>

      <section className="card">
        <div className="filtros">
          <div className="chips">
            <button className="chip" data-on={familia === ""} onClick={() => setFamilia("")}>
              Todas
            </button>
            {familias.map((f) => (
              <button key={f} className="chip" data-on={familia === f} onClick={() => setFamilia(f)}>
                {f}
              </button>
            ))}
          </div>
          <button className="btn btn--sm">
            <IconSliders size={14} />
            Configuração avançada
          </button>
        </div>
      </section>

      <section className="card">
        <div className="card__topo">
          <span className="card__titulo">Decisão</span>
          <span className="card__nota">Diretor e Gerente. Escolhem a linguagem e julgam a peça.</span>
        </div>
        {lista.map((m) => (
          <Linha key={m.tag} m={m} />
        ))}
      </section>
    </>
  );
}

function Linha({ m }: { m: ModeloCatalogo }) {
  return (
    <motion.div
      className="modelo"
      data-fora={!m.supported}
      layout
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28, ease: [0.16, 1, 0.3, 1] }}
    >
      <div className="modelo__esq">
        <MarcaModelo familia={m.family} />
        <div>
          <div className="modelo__id">
            <span className="modelo__nome">{m.label}</span>
            <span className="modelo__tag">{m.tag}</span>
            {m.installed && (
              <span className="tag" data-tone="ok">
                <IconCheck size={11} />
                baixado
              </span>
            )}
            {!m.installed && m.supported && !m.fits_now && (
              <span className="tag" data-tone="warn">
                <span className="tag__dot" />
                não cabe agora
              </span>
            )}
            {m.vision && <span className="tag">enxerga imagem</span>}
          </div>
          <div className="modelo__nota">{m.supported ? m.notes : m.reason}</div>
        </div>
      </div>

      <div className="modelo__dir">
        <div className="modelo__num">
          <span className="modelo__peso">{gb(m.footprint_bytes)}</span>
          <span className="modelo__vel" data-rapido={m.estimated_tps >= 4}>
            ≈ {n1(m.estimated_tps)} tok/s
          </span>
          <span className="modelo__vel">
            {m.moe
              ? `MoE · ${n1(m.active_params_b)}B ativos de ${m.params_b}B`
              : `denso · ${m.params_b}B ativos`}
          </span>
        </div>
        <div className="modelo__acao">
          {m.installed ? (
            <button className="btn btn--quiet btn--sm">
              <IconTrash size={13} />
              Remover
            </button>
          ) : m.supported ? (
            <button className="btn btn--sm">
              <IconDownload size={13} />
              Baixar
            </button>
          ) : null}
        </div>
      </div>
    </motion.div>
  );
}

/* ══ Campanha ══════════════════════════════════════════════════════════════ */

export function TelaCampanha({ ate }: { ate: number }) {
  return (
    <>
      <header className="page__head">
        <h1>Campanha</h1>
        <p>Escreva o que quer atingir. O resto é o revezamento.</p>
      </header>

      <div className="relay-barra">
        <div className="relay-barra__topo">
          <span className="relay-barra__titulo">Trilha de revezamento</span>
          <span className="tag" data-tone={ate >= POSTAS.length ? "ok" : "live"}>
            <span className="tag__dot" />
            {ate >= POSTAS.length ? "concluída" : `turno ${Math.min(ate + 1, POSTAS.length)} de ${POSTAS.length}`}
          </span>
        </div>

        <div
          className="relay"
          data-rede="true"
          style={
            {
              "--postas": POSTAS.length,
              "--percurso": Math.min(ate / Math.max(POSTAS.length - 1, 1), 1),
            } as React.CSSProperties
          }
        >
          {POSTAS.map((p, i) => (
            <div
              key={p.cargo}
              className="posta"
              data-estado={i < ate ? "feito" : i === ate ? "ativo" : "espera"}
            >
              <span className="posta__marca" />
              <span className="posta__cargo">{p.cargo}</span>
              <span className="posta__modelo">{p.modelo}</span>
              <span className="posta__rede">{p.nota}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="split">
        <div className="stack">
          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">Objetivo</span>
            </div>
            <div className="field">
              <span>O que você quer atingir</span>
              <div className="falso-campo">
                Apresentar o torrado novo para quem já compra café em grão e reclama de acidez.
              </div>
            </div>
          </section>

          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">Redes</span>
            </div>
            <div className="stack stack--tight">
              {[
                ["Instagram", "imagem quadrada, legenda curta", true],
                ["LinkedIn", "texto longo, corte em 210 caracteres", false],
              ].map(([nome, nota, on]) => (
                <button key={nome as string} className="choice" data-on={on as boolean}>
                  <span className="choice__marca" aria-hidden />
                  <div>
                    <div className="row row--tight">
                      <span className="choice__title">{nome}</span>
                    </div>
                    <span className="hint">{nota}</span>
                  </div>
                </button>
              ))}
            </div>
          </section>
        </div>

        <div className="split__side">
          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">O que vai acontecer</span>
            </div>
            <div className="auto-grid">
              <Leitura k="turnos de agente" v="9" />
              <Leitura k="imagens" v="4" nota="pelo Gemini" />
              <Leitura k="tempo estimado" v="21" small="min" />
            </div>
            <button className="btn btn--key btn--wide">Rodar campanha</button>
            <p className="hint">
              Estimado pela velocidade medida nesta máquina. O padrão é Simular: monta a
              publicação inteira e para antes do último clique.
            </p>
          </section>
        </div>
      </div>
    </>
  );
}

/* ══ Cérebro ═══════════════════════════════════════════════════════════════ */

export function TelaCerebro() {
  const [no, setNo] = useState<string | null>("publico_alvo");
  const vizinhos = GRAFO.edges
    .filter((e) => e.from === no || e.to === no)
    .map((e) => ({ id: e.from === no ? e.to : e.from, peso: e.weight, tipo: e.type }))
    .sort((a, b) => b.peso - a.peso);

  return (
    <>
      <header className="page__head">
        <h1>Cérebro</h1>
        <p>O contexto que todos os cargos compartilham, em grafo ponderado.</p>
      </header>

      <div className="split">
        <div className="stack">
          <section className="card">
            <div className="grafo-wrap">
              <Grafo grafo={GRAFO} selecionado={no} onSelecionar={setNo} />
            </div>
            <span className="grafo-dica">
              Arraste um node para fixá-lo. Roda do mouse aproxima; duplo clique reenquadra.
            </span>
          </section>
        </div>

        <div className="split__side">
          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">{no ?? "nenhum node"}</span>
              <span className="card__nota">vizinhança ordenada</span>
            </div>
            <p className="hint">
              É exatamente isto que um agente recebe ao consultar: já ordenado por peso e
              cortado por limiar, para o corte não acontecer dentro do modelo.
            </p>
            <table className="data">
              <thead>
                <tr>
                  <th>node</th>
                  <th>relação</th>
                  <th className="n">peso</th>
                </tr>
              </thead>
              <tbody>
                {vizinhos.map((v) => (
                  <tr key={v.id}>
                    <td className="mono">{v.id}</td>
                    <td>{v.tipo}</td>
                    <td className="n">{v.peso.toFixed(2).replace(".", ",")}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>
        </div>
      </div>
    </>
  );
}

function Leitura({ k, v, small, nota }: { k: string; v: string; small?: string; nota?: string }) {
  return (
    <div className="read">
      <span className="read__k">{k}</span>
      <span className="read__v">
        {v}
        {small && <small>{small}</small>}
      </span>
      {nota && <span className="read__note">{nota}</span>}
    </div>
  );
}
