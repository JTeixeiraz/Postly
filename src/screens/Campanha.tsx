import { useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirEstagios } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import { PREFS_VAZIAS, type Credencial, type Diagnostico, type EventoEstagio, type RelatorioCampanha, type ResumoCofre } from "../types";
import Relay, { planejar, postasDeEventos } from "../components/Relay";
import Resultado from "../components/Resultado";
import Referencias from "../components/Referencias";
import type { PastaGaleria } from "../types";
import PlanoExecucao from "../components/PlanoExecucao";
import Selecao from "../components/Selecao";
import { IconArrow, IconCheck, IconSpinner } from "../components/Icons";

export default function Campanha({ diag }: { diag: Diagnostico | null }) {
  const { d, f, idioma } = useIdioma();

  const [cofre, setCofre] = useState<ResumoCofre | null>(null);
  const [prefs, setPrefs] = useState(PREFS_VAZIAS);
  const [chave, setChave] = useState("");
  const [chaveEstado, setChaveEstado] = useState<string | null>(null);

  const [redes, setRedes] = useState<string[]>([]);
  const [credenciais, setCredenciais] = useState<Record<string, Credencial>>({});
  const [salvarCred, setSalvarCred] = useState(false);
  const [objetivo, setObjetivo] = useState("");
  const [qualidade, setQualidade] = useState<"rapida" | "alta">("rapida");
  const [simular, setSimular] = useState(true);
  const [rodadas, setRodadas] = useState(2);
  const [pensar, setPensar] = useState(false);
  const [pasta, setPasta] = useState("");
  const [pastas, setPastas] = useState<PastaGaleria[]>([]);

  const [eventos, setEventos] = useState<EventoEstagio[]>([]);
  const [rodando, setRodando] = useState(false);
  const [relatorio, setRelatorio] = useState<RelatorioCampanha | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const fim = useRef<HTMLDivElement>(null);

  // As pastas da galeria: quem já organizou o material de um produto escolhe
  // a pasta em vez de subir foto por foto de novo.
  useEffect(() => {
    api.galeriaListar().then(setPastas).catch(() => {});
  }, []);

  useEffect(() => {
    void api.resumoCofre().then(setCofre);
    void api.preferencias().then(setPrefs).catch(() => {});
  }, []);

  useEffect(() => {
    let parar: (() => void) | undefined;
    void ouvirEstagios((e) => setEventos((atual) => [...atual, e])).then((x) => (parar = x));
    return () => parar?.();
  }, []);

  useEffect(() => {
    if (rodando) fim.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [eventos.length, rodando]);

  // Antes de rodar, o percurso planejado; durante, o percurso real.
  const postas = useMemo(
    () => (eventos.length > 0 ? postasDeEventos(eventos) : planejar(redes, rodadas)),
    [eventos, redes, rodadas]
  );

  const ultimoDespacho = useMemo(() => {
    for (let i = eventos.length - 1; i >= 0; i--) {
      if (eventos[i].handoff) return eventos[i];
    }
    return null;
  }, [eventos]);

  const emAndamento = eventos.length > 0 ? eventos[eventos.length - 1] : null;

  const salvarChave = async () => {
    setChaveEstado(d.common.loading);
    try {
      setCofre(await api.salvarChaveGemini(chave));
      setChave("");
      await api.validarChaveGemini();
      setChaveEstado(d.campaign.keyValid);
    } catch (e) {
      setChaveEstado(String(e));
    }
  };

  const iniciar = async () => {
    setRodando(true);
    setEventos([]);
    setRelatorio(null);
    setErro(null);
    try {
      setRelatorio(
        await api.iniciarCampanha({
          objetivo,
          redes,
          credenciais,
          salvar_credenciais: salvarCred,
          qualidade_imagem: qualidade,
          simular,
          max_rodadas: rodadas,
          pensamento_estendido: pensar,
          idioma,
        })
      );
    } catch (e) {
      setErro(String(e));
    } finally {
      setRodando(false);
      void api.resumoCofre().then(setCofre);
    }
  };

  if (!diag) return <div className="skeleton" style={{ height: 220 }} />;

  // Primeira vez: sem chave, a campanha nao roda de jeito nenhum. Mostrar os
  // sete cartoes de uma vez faz quem abriu agora ler tudo para descobrir que
  // faltava uma linha. Entao a tela comeca com um passo so.
  const primeiraVez = cofre !== null && !cofre.has_gemini_key;
  const ram = diag.computacao.ram;
  const multi = redes.length > 1;
  const impedimento = !cofre?.has_gemini_key
    ? d.campaign.needKey
    : redes.length === 0
      ? d.campaign.needNetwork
      : objetivo.trim().length < 10
        ? d.campaign.needGoal
        : null;

  return (
    <>
      <header className="page__head">
        <h1>{d.campaign.title}</h1>
        <p>{d.campaign.lead}</p>
      </header>

      {/* ── revezamento ───────────────────────────────────────────────
          Gruda no topo porque cada turno leva minutos: quem volta depois
          de sair da frente da tela procura exatamente isto. */}
      {!primeiraVez && <div className="relay-barra">
        <div className="relay-barra__topo">
          <span className="relay-barra__titulo">{d.campaign.relay}</span>
          {emAndamento && (
            <span className="tag" data-tone={rodando ? "live" : "ok"}>
              <span className="tag__dot" />
              {rotuloEstagio(emAndamento.stage, d)}
            </span>
          )}
          {emAndamento?.available_ram_bytes ? (
            <span className="hint push num">
              {formatarBytes(emAndamento.available_ram_bytes, idioma)} {d.common.free}
            </span>
          ) : null}
        </div>

        <Relay postas={postas} />

        {emAndamento?.detail && (
          <p className="hint" style={{ textAlign: "center" }}>
            {emAndamento.detail}
            {emAndamento.percent !== null && ` \u00b7 ${emAndamento.percent.toFixed(0)}%`}
          </p>
        )}
      </div>}

      <div className="split">
        {/* ── esquerda: o que voce decide, na ordem em que se pensa ──
            Objetivo primeiro. Ele e a campanha; tudo o mais e como e onde.
            Antes ele vinha em quarto lugar, depois de duas telas de
            configuracao, e a pessoa chegava nele ja cansada. */}
        <div>
          {primeiraVez && (
            <section className="card">
              <div className="card__topo">
                <h2>{d.campaign.firstTitle}</h2>
              </div>
              <p>{d.campaign.firstBody}</p>
              <div className="row">
                <input
                  type="password"
                  value={chave}
                  onChange={(e) => setChave(e.target.value)}
                  placeholder={d.campaign.keyPlaceholder}
                  style={{ flex: "1 1 240px" }}
                />
                <button className="btn btn--key" onClick={salvarChave} disabled={chave.trim().length < 10}>
                  {d.common.save}
                </button>
              </div>
              {chaveEstado && <p className="hint">{chaveEstado}</p>}
              <p className="hint">{d.campaign.firstWhere}</p>
            </section>
          )}

          <section className="card">
            <div className="card__topo">
              <h2>{d.campaign.goal}</h2>
            </div>
            <textarea
              value={objetivo}
              onChange={(e) => setObjetivo(e.target.value)}
              placeholder={d.campaign.goalPlaceholder}
              style={{ minHeight: 132 }}
            />
          </section>

          <section className="card">
            <div className="card__topo">
              <h2>{d.campaign.networks}</h2>
              {multi && <span className="hint">{d.campaign.networksMulti}</span>}
            </div>
            <div className="auto-grid">
              {diag.redes_suportadas.map((rede) => {
                const on = redes.includes(rede.slug);
                return (
                  <button
                    key={rede.slug}
                    className="choice"
                    data-on={on}
                    aria-pressed={on}
                    onClick={() =>
                      setRedes((a) => (on ? a.filter((r) => r !== rede.slug) : [...a, rede.slug]))
                    }
                  >
                    <span className="choice__marca" aria-hidden />
                    <div>
                      <div className="row row--tight">
                        <span className="choice__title">{rede.label}</span>
                        {cofre?.saved_networks.includes(rede.slug) && (
                          <span className="tag" data-tone="ok">
                            <span className="tag__dot" />
                            {d.campaign.savedLogin}
                          </span>
                        )}
                      </div>
                      <div className="hint">{rede.formato}</div>
                    </div>
                  </button>
                );
              })}
            </div>
          </section>

          {redes.length > 0 && (
            <section className="card">
              <div className="card__topo">
                <h2>{d.campaign.access}</h2>
              </div>
              <p className="hint" style={{ marginTop: -6 }}>{d.campaign.accessLead}</p>

              <div className="stack">
                {redes.map((slug) => {
                  const rede = diag.redes_suportadas.find((r) => r.slug === slug);
                  const cred = credenciais[slug] ?? { username: "", password: "" };
                  return (
                    <div className="auto-grid" key={slug}>
                      <label className="field">
                        <span>
                          {rede?.label} · {d.campaign.user}
                        </span>
                        <input
                          type="text"
                          autoComplete="off"
                          value={cred.username}
                          onChange={(e) =>
                            setCredenciais((c) => ({ ...c, [slug]: { ...cred, username: e.target.value } }))
                          }
                        />
                      </label>
                      <label className="field">
                        <span>{d.campaign.password}</span>
                        <input
                          type="password"
                          autoComplete="off"
                          value={cred.password}
                          onChange={(e) =>
                            setCredenciais((c) => ({ ...c, [slug]: { ...cred, password: e.target.value } }))
                          }
                        />
                      </label>
                    </div>
                  );
                })}

                <label className="choice" data-on={salvarCred}>
                  <input type="checkbox" checked={salvarCred} onChange={(e) => setSalvarCred(e.target.checked)} />
                  <div>
                    <span className="choice__title">{d.campaign.remember}</span>
                    <div className="hint">{d.campaign.rememberWhy}</div>
                  </div>
                </label>
              </div>
            </section>
          )}

          {pastas.length > 0 && (
            <div className="stack" style={{ marginBottom: "var(--s4)" }}>
              <Selecao
                rotulo={d.galeria.naCampanha}
                valor={pasta}
                opcoes={[
                  { valor: "", rotulo: d.galeria.naCampanhaNenhuma },
                  ...pastas.map((p) => ({
                    valor: p.slug,
                    rotulo: `${p.nome} — ${p.itens.length + p.referencias.length}`,
                  })),
                ]}
                onEscolher={setPasta}
              />
              <p className="hint">{d.galeria.naCampanhaPorque}</p>
            </div>
          )}

          <Referencias prefs={prefs} onMudar={setPrefs} />
        </div>

        {/* ── direita: o que o sistema faz com isso ─────────────────── */}
        <aside className="split__side">
          {/* Chave configurada nao merece um cartao inteiro toda vez: vira uma
              linha, e o espaco fica para o que muda. */}
          {!primeiraVez && cofre?.has_gemini_key && (
            <div className="chave-linha">
              <span className="tag" data-tone="ok">
                <span className="tag__dot" />
                {d.campaign.keySet}
              </span>
              <span className="hint num">{cofre.gemini_key_hint}</span>
              <button
                className="btn btn--quiet btn--sm push"
                onClick={() =>
                  api
                    .validarChaveGemini()
                    .then(() => setChaveEstado(d.campaign.keyValid))
                    .catch((e) => setChaveEstado(String(e)))
                }
              >
                {d.common.test}
              </button>
            </div>
          )}
          {chaveEstado && !primeiraVez && <span className="hint">{chaveEstado}</span>}

          <PlanoExecucao redes={redes} rodadas={rodadas} />

          <section className="card">
            <div className="card__topo">
              <span className="card__titulo">{d.campaign.mode}</span>
            </div>
            <Selecao
              valor={simular ? "s" : "n"}
              onEscolher={(v) => setSimular(v === "s")}
              opcoes={[
                { valor: "s", rotulo: d.campaign.modeDry },
                { valor: "n", rotulo: d.campaign.modeLive },
              ]}
            />
            <div className="auto-grid">
              <label className="field">
                <span>{d.campaign.quality}</span>
                <Selecao
                  valor={qualidade}
                  onEscolher={(v) => setQualidade(v as "rapida" | "alta")}
                  opcoes={[
                    { valor: "rapida", rotulo: d.campaign.qualityFast },
                    { valor: "alta", rotulo: d.campaign.qualityHigh },
                  ]}
                />
              </label>
              <label className="field">
                <span>{d.campaign.rounds}</span>
                <Selecao
                  valor={String(rodadas)}
                  onEscolher={(v) => setRodadas(Number(v))}
                  opcoes={[
                    { valor: "1", rotulo: d.campaign.round1 },
                    { valor: "2", rotulo: d.campaign.round2 },
                    { valor: "3", rotulo: d.campaign.round3 },
                  ]}
                />
              </label>
            </div>

            <label className="choice" data-on={pensar}>
              <input type="checkbox" checked={pensar} onChange={(e) => setPensar(e.target.checked)} />
              <div>
                <span className="choice__title">{d.campaign.think}</span>
                <div className="hint">{d.campaign.thinkWhy}</div>
              </div>
            </label>
          </section>

          <section className="card">
            {ram.low_ram_warning && (
              <span className="tag" data-tone="warn" style={{ justifySelf: "start" }}>
                <span className="tag__dot" />
                {f(d.campaign.lowRam, { free: formatarBytes(ram.available_bytes, idioma) })}
              </span>
            )}
            <button className="btn btn--key btn--wide" onClick={iniciar} disabled={!!impedimento || rodando}>
              {rodando ? <IconSpinner size={16} /> : simular ? <IconCheck size={16} /> : <IconArrow size={16} />}
              {rodando ? d.campaign.running : simular ? d.campaign.startDry : d.campaign.start}
            </button>
            {impedimento && !rodando && <span className="hint">{impedimento}</span>}
            <span className="hint">{d.campaign.costNote}</span>
          </section>

          <AnimatePresence mode="wait">
            {ultimoDespacho?.handoff && (
              <motion.div
                className="despacho"
                key={ultimoDespacho.step}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ type: "spring", stiffness: 240, damping: 26 }}
              >
                <span className="despacho__de">
                  {f(d.campaign.dispatchFrom, { role: ultimoDespacho.role })}
                </span>
                <p className="despacho__texto">{ultimoDespacho.handoff}</p>
              </motion.div>
            )}
          </AnimatePresence>
          <div ref={fim} />
        </aside>
      </div>

      {erro && (
        <div className="note" data-tone="alert" style={{ marginTop: 20 }}>
          <strong>{d.campaign.stopped}</strong>
          <span>{erro}</span>
        </div>
      )}

      {relatorio && <Resultado relatorio={relatorio} />}
    </>
  );
}

function rotuloEstagio(estagio: EventoEstagio["stage"], d: ReturnType<typeof useIdioma>["d"]): string {
  const mapa: Record<EventoEstagio["stage"], string> = {
    medindo_memoria: d.campaign.stageMeasuring,
    escolhendo_modelo: d.campaign.stagePicking,
    baixando_modelo: d.campaign.stageDownloading,
    pensando: d.campaign.stageThinking,
    descarregando: d.campaign.stageUnloading,
    concluido: d.campaign.stageDone,
    falhou: d.campaign.stageFailed,
  };
  return mapa[estagio];
}
