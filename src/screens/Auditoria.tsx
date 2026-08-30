import { useCallback, useEffect, useMemo, useState } from "react";
import { motion } from "motion/react";
import { api } from "../api";
import { formatarNumero, useIdioma } from "../i18n";
import type { LeituraDaRede, RegistroMetrica } from "../types";
import { IconArrow, IconCheck, IconTrash } from "../components/Icons";
import Porque from "../components/Porque";
import Selecao from "../components/Selecao";

const REDES = ["instagram", "facebook", "tiktok", "linkedin", "x"];

const ROTULO_REDE: Record<string, string> = {
  instagram: "Instagram",
  facebook: "Facebook",
  tiktok: "TikTok",
  linkedin: "LinkedIn",
  x: "X",
};

/** Campos numericos do formulario, na ordem em que se le um painel de rede. */
const NUMEROS = [
  { chave: "impressoes", rotulo: "reach", ajuda: "reachHelp" },
  { chave: "curtidas", rotulo: "likes" },
  { chave: "comentarios", rotulo: "comments" },
  { chave: "compartilhamentos", rotulo: "shares" },
  { chave: "salvamentos", rotulo: "saves" },
  { chave: "cliques", rotulo: "clicks" },
] as const;

const VAZIO: Partial<RegistroMetrica> = {
  rede: "instagram",
  publicado_em: new Date().toISOString().slice(0, 10),
  conceito: "",
  url: "",
  impressoes: 0,
  curtidas: 0,
  comentarios: 0,
  compartilhamentos: 0,
  salvamentos: 0,
  cliques: 0,
};

export default function Auditoria() {
  const { d, f, idioma } = useIdioma();
  const [regs, setRegs] = useState<RegistroMetrica[] | null>(null);
  const [leituras, setLeituras] = useState<LeituraDaRede[]>([]);
  const [form, setForm] = useState<Partial<RegistroMetrica> | null>(null);
  const [analise, setAnalise] = useState<string | null>(null);
  const [ocupado, setOcupado] = useState<string | null>(null);
  const [recado, setRecado] = useState<string | null>(null);
  const [redeColeta, setRedeColeta] = useState("instagram");

  const recarregar = useCallback(async () => {
    setRegs(await api.listarMetricas().catch(() => []));
    setLeituras(await api.leituraDesempenho().catch(() => []));
  }, []);

  useEffect(() => {
    void recarregar();
  }, [recarregar]);

  const salvar = async () => {
    if (!form) return;
    setRecado(null);
    try {
      setRegs(await api.registrarMetrica(form));
      setLeituras(await api.leituraDesempenho());
      setForm(null);
    } catch (e) {
      setRecado(String(e));
    }
  };

  const remover = async (id: string) => {
    setRegs(await api.removerMetrica(id).catch(() => regs ?? []));
    setLeituras(await api.leituraDesempenho().catch(() => []));
    setRecado(d.audit.removed);
  };

  const coletar = async (rede: string) => {
    setOcupado("coleta");
    setRecado(null);
    try {
      setRegs(await api.coletarMetricas(rede, 12));
      setLeituras(await api.leituraDesempenho());
    } catch (e) {
      setRecado(String(e));
    } finally {
      setOcupado(null);
    }
  };

  const analisar = async () => {
    setOcupado("analise");
    setRecado(null);
    try {
      setAnalise(await api.analisarDesempenho());
    } catch (e) {
      setRecado(String(e));
    } finally {
      setOcupado(null);
    }
  };

  const porRede = useMemo(() => {
    const mapa = new Map<string, RegistroMetrica[]>();
    for (const r of regs ?? []) {
      mapa.set(r.rede, [...(mapa.get(r.rede) ?? []), r]);
    }
    return mapa;
  }, [regs]);

  if (!regs) {
    return (
      <div className="stack">
        <div className="skeleton" style={{ height: 30, width: 210 }} />
        <div className="skeleton" style={{ height: 120 }} />
      </div>
    );
  }

  return (
    <>
      <header className="page__head">
        <h1>{d.audit.title}</h1>
        <p>{d.audit.lead}</p>
        <Porque>{d.audit.why}</Porque>
      </header>

      <div className="row" style={{ marginBottom: 4 }}>
        <button className="btn btn--key" onClick={() => setForm({ ...VAZIO })}>
          {d.audit.add}
        </button>
        <div className="coleta">
          <Selecao
            valor={redeColeta}
            opcoes={REDES.map((r) => ({ valor: r, rotulo: ROTULO_REDE[r] }))}
            onEscolher={setRedeColeta}
          />
          <button
            className="btn"
            disabled={ocupado !== null}
            onClick={() => void coletar(redeColeta)}
            title={d.audit.collectWhy}
          >
            {ocupado === "coleta" ? d.audit.collecting : d.audit.collect}
          </button>
        </div>
        <button
          className="btn push"
          disabled={ocupado !== null || regs.length < 3}
          onClick={() => void analisar()}
        >
          {ocupado === "analise" ? d.audit.analysing : d.audit.analyse}
          <IconArrow size={14} />
        </button>
      </div>

      {recado && (
        <div className="note" data-tone="signal">
          <span>{recado}</span>
        </div>
      )}

      {form && (
        <motion.section
          className="card"
          initial={{ opacity: 0, y: -6 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div className="card__topo">
            <span className="card__titulo">{d.audit.add}</span>
            <button className="btn btn--quiet btn--sm push" onClick={() => setForm(null)}>
              {d.common.close}
            </button>
          </div>

          <div className="grade">
            <label className="field g4">
              <span>{d.audit.network}</span>
              <Selecao
                valor={form.rede ?? "instagram"}
                opcoes={REDES.map((r) => ({ valor: r, rotulo: ROTULO_REDE[r] }))}
                onEscolher={(v) => setForm({ ...form, rede: v })}
              />
            </label>
            <label className="field g4">
              <span>{d.audit.date}</span>
              <input
                type="text"
                value={form.publicado_em ?? ""}
                placeholder="2026-08-29"
                onChange={(e) => setForm({ ...form, publicado_em: e.target.value })}
              />
            </label>
            <label className="field g4">
              <span>{d.audit.url}</span>
              <input
                type="text"
                value={form.url ?? ""}
                onChange={(e) => setForm({ ...form, url: e.target.value })}
              />
              <span className="field__help">{d.audit.urlHelp}</span>
            </label>

            <label className="field g12">
              <span>{d.audit.concept}</span>
              <input
                type="text"
                value={form.conceito ?? ""}
                onChange={(e) => setForm({ ...form, conceito: e.target.value })}
              />
              <span className="field__help">{d.audit.conceptHelp}</span>
            </label>

            {NUMEROS.map((n) => (
              <label className="field g4" key={n.chave}>
                <span>{d.audit[n.rotulo]}</span>
                <input
                  type="text"
                  inputMode="numeric"
                  className="num"
                  value={String(form[n.chave] ?? 0)}
                  onChange={(e) =>
                    setForm({
                      ...form,
                      [n.chave]: Number(e.target.value.replace(/\D/g, "")) || 0,
                    })
                  }
                />
                {"ajuda" in n && <span className="field__help">{d.audit[n.ajuda]}</span>}
              </label>
            ))}
          </div>

          <button className="btn btn--key" onClick={() => void salvar()}>
            <IconCheck size={14} />
            {d.audit.save}
          </button>
        </motion.section>
      )}

      {analise && (
        <motion.section
          className="card card--light"
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div className="card__topo">
            <span className="card__titulo">{d.audit.analysisTitle}</span>
            <button className="btn btn--quiet btn--sm push" onClick={() => setAnalise(null)}>
              {d.common.close}
            </button>
          </div>
          <pre className="raw" style={{ background: "transparent", border: 0, padding: 0 }}>
            {analise}
          </pre>
        </motion.section>
      )}

      {regs.length === 0 ? (
        <div className="empty">
          <h3>{d.audit.empty}</h3>
          <p>{d.audit.emptyWhy}</p>
        </div>
      ) : (
        leituras.map((l) => (
          <Rede
            key={l.rede}
            leitura={l}
            registros={porRede.get(l.rede) ?? []}
            onRemover={remover}
            idioma={idioma}
            d={d}
            f={f}
          />
        ))
      )}
    </>
  );
}

function Rede({
  leitura,
  registros,
  onRemover,
  idioma,
  d,
  f,
}: {
  leitura: LeituraDaRede;
  registros: RegistroMetrica[];
  onRemover: (id: string) => void;
  idioma: "pt" | "en";
  // Os dicionarios sao grandes e mudam a cada feature; tipa-los aqui de novo
  // duplicaria a fonte da verdade, que e o proprio pt.ts.
  d: any;
  f: (t: string, v: Record<string, string | number>) => string;
}) {
  const veredito = {
    sem_base: { rotulo: d.audit.verdictNone, porque: d.audit.verdictNoneWhy, tom: undefined },
    divergir: {
      rotulo: d.audit.verdictDiverge,
      porque: f(d.audit.verdictDivergeWhy, { n: formatarNumero(leitura.mediana, idioma, 1) }),
      tom: "live",
    },
    seguir: {
      rotulo: d.audit.verdictFollow,
      porque: f(d.audit.verdictFollowWhy, {
        c: leitura.melhor_conceito,
        x: formatarNumero(leitura.multiplo_da_melhor, idioma, 1),
      }),
      tom: "ok",
    },
  }[leitura.veredito];

  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{ROTULO_REDE[leitura.rede] ?? leitura.rede}</span>
        <span className="tag" data-tone={veredito.tom}>
          <span className="tag__dot" />
          {veredito.rotulo}
        </span>
        <span className="card__nota">
          {leitura.publicacoes} {d.audit.posts} · {d.audit.median}{" "}
          <b>{formatarNumero(leitura.mediana, idioma, 1)}</b>
          <br />
          {leitura.base === "taxa" ? d.audit.basisRate : d.audit.basisVolume}
        </span>
      </div>

      <p className="hint">{veredito.porque}</p>

      {leitura.base === "volume" && (
        <div className="note" data-tone="warn">
          <span>{d.audit.basisWarn}</span>
        </div>
      )}
      {leitura.fora_da_base > 0 && (
        <p className="hint">{f(d.audit.outOfBasis, { n: leitura.fora_da_base })}</p>
      )}

      {/* Barra proporcional ao multiplo: a diferenca entre 3,1x e 0,4x se ve
          antes de se ler, e e essa comparacao que decide a proxima campanha. */}
      <div className="stack stack--tight">
        {leitura.ranking.map((item) => {
          const reg = registros.find((r) => r.id === item.id);
          const largura = Math.min(item.multiplo / Math.max(leitura.multiplo_da_melhor, 1), 1) * 100;
          return (
            <div className="linha-peca" key={item.id}>
              <div className="linha-peca__topo">
                <span className="linha-peca__conceito">{item.conceito}</span>
                <span className="linha-peca__mult" data-forte={item.multiplo >= 1}>
                  {formatarNumero(item.multiplo, idioma, 2)}×
                </span>
                {reg && (
                  <button
                    className="btn btn--quiet btn--sm"
                    onClick={() => onRemover(reg.id)}
                    aria-label={d.common.remove}
                  >
                    <IconTrash size={13} />
                  </button>
                )}
              </div>
              <div className="linha-peca__barra">
                <div style={{ width: `${largura}%` }} data-forte={item.multiplo >= 1} />
              </div>
              <div className="linha-peca__pe">
                {item.publicado_em && <span>{item.publicado_em}</span>}
                {reg && (
                  <>
                    <span className="num">
                      {reg.curtidas} {d.audit.likes.toLowerCase()}
                    </span>
                    <span className="num">
                      {reg.comentarios} {d.audit.comments.toLowerCase()}
                    </span>
                    {item.impressoes > 0 && (
                      <span className="num">
                        {item.impressoes} {d.audit.reach.toLowerCase()}
                      </span>
                    )}
                    <span className="push">
                      {reg.origem === "raspagem" ? d.audit.fromScrape : d.audit.fromManual}
                    </span>
                  </>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
