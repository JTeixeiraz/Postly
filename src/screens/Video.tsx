import { useCallback, useEffect, useState } from "react";
import { api, ouvirRender } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type {
  NotaDeCena,
  ProgressoRender,
  ProjetoVideo,
  RelatorioVideo,
} from "../types";
import AssetsVideo from "../components/AssetsVideo";
import RoteiroPronto from "../components/RoteiroPronto";
import Monitor from "../components/Monitor";
import Linha from "../components/Linha";
import Inspetor from "../components/Inspetor";


type Aba = "montagem" | "assets" | "briefing";

const PROPORCOES = ["16:9", "9:16", "1:1"];

/** O editor de vídeo em que a pessoa não edita.
 *
 *  A FORMA É DE EDITOR — monitor em cima, régua e trilhas embaixo, inspetor na
 *  lateral — e a função não é. Não há alça para arrastar, corte, nem keyframe:
 *  quem monta é o Motion Designer. O que a forma dá é **vocabulário para
 *  apontar**. Uma lista de cenas em texto não mostra ritmo; blocos
 *  proporcionais à duração mostram num relance que a cena 3 tem o dobro da 2 —
 *  que é o tipo de coisa que a pessoa julga bem e o modelo erra.
 *
 *  A única ação de edição é a nota: seleciona a cena, para o vídeo no instante,
 *  escreve o que está errado. "Refazer" manda tudo de volta para o cargo, com o
 *  roteiro anterior junto, para ele corrigir em vez de recomeçar. */
export default function Video() {
  const { d, f, idioma } = useIdioma();
  const [projetos, setProjetos] = useState<ProjetoVideo[] | null>(null);
  const [atual, setAtual] = useState<ProjetoVideo | null>(null);
  const [aba, setAba] = useState<Aba>("montagem");

  const [objetivo, setObjetivo] = useState("");
  const [proporcao, setProporcao] = useState("16:9");
  const [pensar, setPensar] = useState(false);

  const [rodando, setRodando] = useState(false);
  const [render, setRender] = useState<ProgressoRender | null>(null);
  const [relatorio, setRelatorio] = useState<RelatorioVideo | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  // Estado da bancada de edição.
  const [selecionada, setSelecionada] = useState<number | null>(null);
  const [segundo, setSegundo] = useState(0);
  const [buscarPara, setBuscarPara] = useState<number | null>(null);
  const [notas, setNotas] = useState<NotaDeCena[]>([]);

  const recarregar = useCallback(async () => {
    const lista = await api.videoListar().catch(() => []);
    setProjetos(lista);
    // Mantém o projeto aberto entre recargas: perder a seleção depois de subir
    // um arquivo mandaria a pessoa de volta para a lista a cada upload.
    setAtual((a) => (a ? (lista.find((p) => p.slug === a.slug) ?? null) : null));
  }, []);

  useEffect(() => {
    void recarregar();
  }, [recarregar]);

  useEffect(() => {
    let parar: (() => void) | undefined;
    void ouvirRender(setRender).then((x) => (parar = x));
    return () => parar?.();
  }, []);

  const criar = async () => {
    const nome = prompt(d.video.newName);
    if (!nome?.trim()) return;
    try {
      const p = await api.videoCriar(nome);
      await recarregar();
      setAtual(p);
      setErro(null);
    } catch (e) {
      setErro(String(e));
    }
  };

  /** Roda do zero, ou refaz com as notas.
   *
   *  Um caminho só: a diferença entre gerar e refazer é o que vai no pedido, e
   *  dois botões chamando funções diferentes acabariam divergindo no dia em que
   *  um deles ganhasse um campo novo. */
  const gerar = async (comNotas: boolean) => {
    if (!atual) return;
    setRodando(true);
    setErro(null);
    setRender(null);
    setAba("montagem");
    try {
      const r = await api.videoGerar({
        projeto: atual.slug,
        objetivo,
        proporcao,
        idioma,
        pensamento_estendido: pensar,
        notas: comNotas ? notas : [],
        roteiro_anterior: comNotas ? (relatorio?.roteiro ?? null) : null,
        linha_anterior: comNotas ? (relatorio?.linha ?? "") : "",
      });
      setRelatorio(r);
      // As notas atendidas somem: mantê-las na tela faria a pessoa mandar de
      // novo, na rodada seguinte, uma correção que o cargo já fez.
      if (comNotas) setNotas([]);
      setSelecionada(null);
      setSegundo(0);
      await recarregar();
    } catch (e) {
      setErro(String(e));
    } finally {
      setRodando(false);
      setRender(null);
    }
  };

  if (!projetos) return <div className="skeleton" style={{ height: 200 }} />;

  if (!atual) {
    return (
      <section className="card">
        <div className="card__topo">
          <h2>{d.video.title}</h2>
          <button className="btn btn--key" onClick={() => void criar()}>
            {d.video.new}
          </button>
        </div>
        <p className="hint">{d.video.lead}</p>

        <div className="stack stack--tight">
          {projetos.map((p) => (
            <button className="choice" key={p.slug} onClick={() => setAtual(p)}>
              <span className="choice__marca" aria-hidden />
              <div>
                <span className="choice__title">{p.nome}</span>
                <div className="hint">
                  {f(d.video.counts, {
                    i: p.imagens.length,
                    a: p.audio.length,
                    v: p.narracao.length,
                  })}
                  {" · "}
                  {formatarBytes(p.bytes, idioma)}
                </div>
              </div>
            </button>
          ))}
          {!projetos.length && <p className="hint">{d.video.emptyList}</p>}
        </div>

        {erro && (
          <div className="note" data-tone="alert">
            <span>{erro}</span>
          </div>
        )}
      </section>
    );
  }

  const abas: { id: Aba; rotulo: string }[] = [
    { id: "montagem", rotulo: d.video.tabEdit },
    { id: "assets", rotulo: d.video.tabAssets },
    { id: "briefing", rotulo: d.video.tabBrief },
  ];

  return (
    <>
      <section className="card card--flat">
        <div className="card__topo">
          <button className="btn btn--quiet btn--sm" onClick={() => setAtual(null)}>
            {d.common.back}
          </button>
          <h2>{atual.nome}</h2>
          <div className="chips">
            {abas.map((a) => (
              <button
                key={a.id}
                className="chip"
                data-on={aba === a.id}
                aria-pressed={aba === a.id}
                onClick={() => setAba(a.id)}
              >
                {a.rotulo}
              </button>
            ))}
          </div>
        </div>
      </section>

      {aba === "assets" && <AssetsVideo projeto={atual} aoMudar={setAtual} />}

      {aba === "briefing" && (
        <Briefing
          projeto={atual}
          objetivo={objetivo}
          setObjetivo={setObjetivo}
          proporcao={proporcao}
          setProporcao={setProporcao}
          pensar={pensar}
          setPensar={setPensar}
          rodando={rodando}
          aoGerar={() => void gerar(false)}
        />
      )}

      {aba === "montagem" && (
        <>
          <div className="bancada">
            <Monitor
              video={relatorio?.video ?? null}
              segundo={segundo}
              aoAndar={setSegundo}
              render={render}
              buscarPara={buscarPara}
            />
            {relatorio?.roteiro && (
              <Inspetor
                roteiro={relatorio.roteiro}
                indice={selecionada}
                segundo={segundo}
                notas={notas}
                aoAnotar={setNotas}
              />
            )}
          </div>

          {relatorio?.roteiro && (
            <section className="card card--flat">
              <Linha
                roteiro={relatorio.roteiro}
                selecionada={selecionada}
                aoSelecionar={setSelecionada}
                segundo={segundo}
                aoBuscar={(s) => {
                  setSegundo(s);
                  // Objeto novo a cada busca: dois cliques no mesmo instante
                  // precisam mover o vídeo as duas vezes, e um número igual não
                  // dispara o efeito do monitor.
                  setBuscarPara(s + Math.random() * 1e-6);
                }}
                narracao={atual.narracao.length}
                temTrilha={!!relatorio.roteiro.trilha}
              />

              <div className="row">
                <button
                  className="btn btn--key"
                  disabled={rodando || !notas.length}
                  onClick={() => void gerar(true)}
                >
                  {f(d.video.redo, { n: notas.length })}
                </button>
                {/* O que "refazer" faz, dito antes do clique: ele corrige, não
                    recomeça. Sem esta frase a pessoa esperaria um vídeo novo. */}
                <span className="hint">
                  {notas.length ? d.video.redoWhat : d.video.redoHow}
                </span>
              </div>
            </section>
          )}

          <Resultado
            projeto={atual}
            rodando={rodando}
            relatorio={relatorio}
            erro={erro}
          />
        </>
      )}
    </>
  );
}

function Briefing({
  projeto,
  objetivo,
  setObjetivo,
  proporcao,
  setProporcao,
  pensar,
  setPensar,
  rodando,
  aoGerar,
}: {
  projeto: ProjetoVideo;
  objetivo: string;
  setObjetivo: (s: string) => void;
  proporcao: string;
  setProporcao: (s: string) => void;
  pensar: boolean;
  setPensar: (b: boolean) => void;
  rodando: boolean;
  aoGerar: () => void;
}) {
  const { d } = useIdioma();
  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{d.video.brief}</span>
      </div>

      <label className="field">
        <span>{d.video.goal}</span>
        <textarea
          rows={4}
          value={objetivo}
          placeholder={d.video.goalPlaceholder}
          onChange={(e) => setObjetivo(e.target.value)}
        />
        <span className="field__help">{d.video.goalHint}</span>
      </label>

      <div className="field">
        <span>{d.video.ratio}</span>
        <div className="chips">
          {PROPORCOES.map((p) => (
            <button
              key={p}
              className="chip"
              data-on={proporcao === p}
              aria-pressed={proporcao === p}
              onClick={() => setProporcao(p)}
            >
              {p}
            </button>
          ))}
        </div>
      </div>

      <label className="row">
        <input type="checkbox" checked={pensar} onChange={(e) => setPensar(e.target.checked)} />
        <span>{d.video.think}</span>
      </label>

      {/* O que vai acontecer, dito ANTES do clique — inclusive a parte que
          costuma surpreender: se não há narração na pasta, o vídeo vai parar no
          meio para perguntar. */}
      <div className="note" data-tone={projeto.narracao.length ? "ok" : "warn"}>
        <span>{projeto.narracao.length ? d.video.willUseVoice : d.video.willAsk}</span>
      </div>

      <button
        className="btn btn--key"
        disabled={rodando || objetivo.trim().length < 10 || !projeto.imagens.length}
        onClick={aoGerar}
      >
        {rodando ? d.video.running : d.video.generate}
      </button>

      {!projeto.imagens.length && <p className="hint">{d.video.needImages}</p>}
    </section>
  );
}

function Resultado({
  projeto,
  rodando,
  relatorio,
  erro,
}: {
  projeto: ProjetoVideo;
  rodando: boolean;
  relatorio: RelatorioVideo | null;
  erro: string | null;
}) {
  const { d, idioma } = useIdioma();

  return (
    <>
      {erro && (
        <div className="note" data-tone="alert">
          <span>{erro}</span>
        </div>
      )}

      {relatorio?.locucao && <RoteiroPronto locucao={relatorio.locucao} />}

      {relatorio?.parecer && (
        <div className="note" data-tone={relatorio.aprovado ? "ok" : "warn"}>
          <span>
            <strong>{d.video.review}: </strong>
            {relatorio.parecer}
          </span>
        </div>
      )}

      {relatorio?.avisos.map((a, i) => (
        <div className="note" data-tone="warn" key={i}>
          <span>{a}</span>
        </div>
      ))}

      {relatorio?.video && (
        <div className="row">
          <button
            className="btn"
            onClick={() => void api.abrirNoSistema(relatorio.video!.arquivo)}
          >
            {d.video.open}
          </button>
          <span className="hint num">
            {formatarBytes(relatorio.video.bytes, idioma)} · {relatorio.video.arquivo}
          </span>
        </div>
      )}

      {/* Os vídeos anteriores continuam na pasta. Sem esta lista, refazer
          pareceria ter apagado o de antes. */}
      {!!projeto.saidas.length && (
        <section className="card">
          <div className="card__topo">
            <span className="card__titulo">{d.video.previous}</span>
            <span className="tag">{projeto.saidas.length}</span>
          </div>
          <div className="stack stack--tight">
            {projeto.saidas.map((s) => (
              <div className="chave-linha" key={s.caminho}>
                <span style={{ flex: 1 }}>{s.nome}</span>
                <span className="hint num">{formatarBytes(s.bytes, idioma)}</span>
                <button
                  className="btn btn--quiet btn--sm"
                  onClick={() => void api.abrirNoSistema(s.caminho)}
                >
                  {d.video.open}
                </button>
              </div>
            ))}
          </div>
        </section>
      )}

      {!rodando && !relatorio && !erro && <p className="hint">{d.video.nothingYet}</p>}
    </>
  );
}
