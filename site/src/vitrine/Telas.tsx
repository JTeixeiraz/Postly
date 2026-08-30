import { useMemo, useState } from "react";
import { motion } from "motion/react";
import MarcaModelo from "../app/MarcaModelo";
import Grafo from "../app/Grafo";
import { IconCheck, IconDownload, IconSliders, IconTrash } from "../app/Icons";
import { grafoDe, MODELOS, MODELOS_DAS_POSTAS } from "./dados";
import type { ModeloCatalogo } from "./tipos";
import { useIdioma, type Idioma } from "../i18n";

/** Português escreve 9,3; inglês escreve 9.3. Um número decimal com o
 *  separador errado lê como outro número, então a formatação acompanha o
 *  idioma junto com o texto. */
const dec = (v: string, i: Idioma) => (i === "pt" ? v.replace(".", ",") : v);
const gb = (b: number, i: Idioma) => `${dec((b / 1e9).toFixed(1), i)} GB`;
const n1 = (v: number, i: Idioma) => dec(v.toFixed(1), i);

/* ══ Modelos ═══════════════════════════════════════════════════════════════ */

export function TelaModelos() {
  const { d, idioma } = useIdioma();
  const t = d.vitrine.modelos;
  const [familia, setFamilia] = useState("");
  const familias = [...new Set(MODELOS.map((m) => m.family))];
  const lista = MODELOS.filter((m) => !familia || m.family === familia);

  return (
    <>
      <header className="page__head">
        <h1>{t.titulo}</h1>
        <p>{t.texto}</p>
      </header>

      <section className="card">
        <div className="auto-grid">
          <Leitura k={t.teto} v={gb(21_500_000_000, idioma)} nota={t.tetoNota} />
          <Leitura k={t.cabe} v="7" small="/ 8" nota={t.cabeNota} />
          <Leitura k={t.baixado} v="2" nota={gb(23_900_000_000, idioma)} />
        </div>
      </section>

      <section className="card">
        <div className="filtros">
          <div className="chips">
            <button className="chip" data-on={familia === ""} onClick={() => setFamilia("")}>
              {t.todas}
            </button>
            {familias.map((f) => (
              <button key={f} className="chip" data-on={familia === f} onClick={() => setFamilia(f)}>
                {f}
              </button>
            ))}
          </div>
          <button className="btn btn--sm">
            <IconSliders size={14} />
            {t.avancado}
          </button>
        </div>
      </section>

      <section className="card">
        <div className="card__topo">
          <span className="card__titulo">{t.decisao}</span>
          <span className="card__nota">{t.decisaoNota}</span>
        </div>
        {lista.map((m) => (
          <Linha key={m.tag} m={m} />
        ))}
      </section>
    </>
  );
}

function Linha({ m }: { m: ModeloCatalogo }) {
  const { d, idioma } = useIdioma();
  const t = d.vitrine.modelos;
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
                {t.tagBaixado}
              </span>
            )}
            {!m.installed && m.supported && !m.fits_now && (
              <span className="tag" data-tone="warn">
                <span className="tag__dot" />
                {t.tagNaoCabe}
              </span>
            )}
            {m.vision && <span className="tag">{t.tagVisao}</span>}
          </div>
          <div className="modelo__nota">{m.supported ? t.notas[m.tag] : t.razoes[m.tag]}</div>
        </div>
      </div>

      <div className="modelo__dir">
        <div className="modelo__num">
          <span className="modelo__peso">{gb(m.footprint_bytes, idioma)}</span>
          <span className="modelo__vel" data-rapido={m.estimated_tps >= 4}>
            ≈ {n1(m.estimated_tps, idioma)} tok/s
          </span>
          <span className="modelo__vel">
            {m.moe
              ? t.moe(n1(m.active_params_b, idioma), m.params_b)
              : t.denso(String(m.params_b))}
          </span>
        </div>
        <div className="modelo__acao">
          {m.installed ? (
            <button className="btn btn--quiet btn--sm">
              <IconTrash size={13} />
              {t.remover}
            </button>
          ) : m.supported ? (
            <button className="btn btn--sm">
              <IconDownload size={13} />
              {t.baixar}
            </button>
          ) : null}
        </div>
      </div>
    </motion.div>
  );
}

/* ══ Campanha ══════════════════════════════════════════════════════════════ */

export function TelaCampanha({ ate }: { ate: number }) {
  const { d } = useIdioma();
  const t = d.vitrine.campanha;
  const postas = t.postas.map((p, i) => ({ ...p, modelo: MODELOS_DAS_POSTAS[i] }));
  return (
    <>
      <header className="page__head">
        <h1>{t.titulo}</h1>
        <p>{t.texto}</p>
      </header>

      <div className="relay-barra">
        <div className="relay-barra__topo">
          <span className="relay-barra__titulo">{t.trilha}</span>
          <span className="tag" data-tone={ate >= postas.length ? "ok" : "live"}>
            <span className="tag__dot" />
            {ate >= postas.length
              ? t.concluida
              : t.turno(Math.min(ate + 1, postas.length), postas.length)}
          </span>
        </div>

        <div
          className="relay"
          data-rede="true"
          style={
            {
              "--postas": postas.length,
              "--percurso": Math.min(ate / Math.max(postas.length - 1, 1), 1),
            } as React.CSSProperties
          }
        >
          {postas.map((p, i) => (
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
              <span className="card__titulo">{t.objetivo}</span>
            </div>
            <div className="field">
              <span>{t.objetivoRotulo}</span>
              <div className="falso-campo">{t.objetivoTexto}</div>
            </div>
          </section>

          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">{t.redes}</span>
            </div>
            <div className="stack stack--tight">
              {[
                ["Instagram", t.instagramNota, true],
                ["LinkedIn", t.linkedinNota, false],
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
              <span className="card__titulo">{t.previsao}</span>
            </div>
            <div className="auto-grid">
              <Leitura k={t.turnos} v="9" />
              <Leitura k={t.imagens} v="4" nota={t.imagensNota} />
              <Leitura k={t.tempo} v="21" small={t.min} />
            </div>
            <button className="btn btn--key btn--wide">{t.rodar}</button>
            <p className="hint">{t.rodape}</p>
          </section>
        </div>
      </div>
    </>
  );
}

/* ══ Cérebro ═══════════════════════════════════════════════════════════════ */

export function TelaCerebro() {
  const { d, idioma } = useIdioma();
  const t = d.vitrine.cerebro;
  // O grafo é remontado a cada idioma porque os ids são o rótulo desenhado.
  const grafo = useMemo(() => grafoDe(t.nodes, t.relacoes), [t]);
  const [no, setNo] = useState<string | null>(null);
  // O node em foco segue o idioma: guardar "publico_alvo" e trocar para o
  // inglês deixaria a seleção apontando para um node que não existe mais.
  const foco = no ?? t.nodes.publico_alvo;
  const vizinhos = grafo.edges
    .filter((e) => e.from === foco || e.to === foco)
    .map((e) => ({ id: e.from === foco ? e.to : e.from, peso: e.weight, tipo: e.type }))
    .sort((a, b) => b.peso - a.peso);

  return (
    <>
      <header className="page__head">
        <h1>{t.titulo}</h1>
        <p>{t.texto}</p>
      </header>

      <div className="split">
        <div className="stack">
          <section className="card">
            <div className="grafo-wrap">
              <Grafo grafo={grafo} selecionado={foco} onSelecionar={setNo} />
            </div>
            <span className="grafo-dica">{t.dica}</span>
          </section>
        </div>

        <div className="split__side">
          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">{foco || t.nenhum}</span>
              <span className="card__nota">{t.vizinhanca}</span>
            </div>
            <p className="hint">{t.explicacao}</p>
            <table className="data">
              <thead>
                <tr>
                  <th>{t.colNode}</th>
                  <th>{t.colRelacao}</th>
                  <th className="n">{t.colPeso}</th>
                </tr>
              </thead>
              <tbody>
                {vizinhos.map((v) => (
                  <tr key={v.id}>
                    <td className="mono">{v.id}</td>
                    <td>{v.tipo}</td>
                    <td className="n">{dec(v.peso.toFixed(2), idioma)}</td>
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
