import { useCallback, useEffect, useMemo, useState } from "react";
import { api, ouvirDownloads } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import {
  PREFS_VAZIAS,
  type Diagnostico,
  type Nivel,
  type Preferencias,
  type StatusProvedor,
} from "../types";
import { IconArrow, IconSliders } from "../components/Icons";
import LinhaModelo from "../components/LinhaModelo";
import CargosAvancado from "../components/CargosAvancado";
import Porque from "../components/Porque";
import Provedor from "../components/Provedor";
import ElencoClaude from "../components/ElencoClaude";

export default function Modelos({
  diag,
  avancar,
}: {
  diag: Diagnostico | null;
  avancar: () => void;
}) {
  const { d, f, idioma } = useIdioma();
  const [modelos, setModelos] = useState<import("../types").ModeloCatalogo[] | null>(null);
  const [prefs, setPrefs] = useState<Preferencias>(PREFS_VAZIAS);
  const [familia, setFamilia] = useState<string>("");
  const [verFora, setVerFora] = useState(false);
  const [baixando, setBaixando] = useState<Record<string, number>>({});
  const [removendo, setRemovendo] = useState<string | null>(null);
  const [recado, setRecado] = useState<string | null>(null);
  const [provedor, setProvedor] = useState<StatusProvedor | null>(null);

  const recarregar = useCallback(async () => {
    setModelos(await api.catalogoModelos().catch(() => []));
  }, []);

  useEffect(() => {
    void recarregar();
    void api.preferencias().then(setPrefs).catch(() => {});
  }, [recarregar]);

  // O Ollama informa o andamento do pull; sem isto o botao ficaria parado por
  // minutos num download de 20 GB, e a pessoa nao saberia se travou.
  useEffect(() => {
    let parar: (() => void) | undefined;
    void ouvirDownloads((e) => {
      setBaixando((atual) => ({ ...atual, [e.model]: e.percent }));
    }).then((x) => (parar = x));
    return () => parar?.();
  }, []);

  const baixar = async (tag: string) => {
    setRecado(null);
    setBaixando((a) => ({ ...a, [tag]: 0 }));
    try {
      await api.baixarModelo(tag);
      await recarregar();
    } catch (e) {
      setRecado(f(d.models.downloadFailed, { tag, erro: String(e) }));
    } finally {
      setBaixando((a) => {
        const resto = { ...a };
        delete resto[tag];
        return resto;
      });
    }
  };

  const remover = async (tag: string) => {
    setRecado(null);
    setRemovendo(tag);
    try {
      await api.removerModelo(tag);
      setRecado(f(d.models.removed, { tag }));
      await recarregar();
    } catch (e) {
      setRecado(String(e));
    } finally {
      setRemovendo(null);
    }
  };

  const familias = useMemo(
    () => [...new Set((modelos ?? []).map((m) => m.family))].sort(),
    [modelos]
  );

  const visiveis = useMemo(
    () => (modelos ?? []).filter((m) => !familia || m.family === familia),
    [modelos, familia]
  );

  const niveis = useMemo(
    () =>
      [
        { id: "alto" as Nivel, titulo: d.models.tierHigh, quem: d.models.tierHighWho },
        { id: "medio" as Nivel, titulo: d.models.tierMid, quem: d.models.tierMidWho },
        { id: "baixo" as Nivel, titulo: d.models.tierLow, quem: d.models.tierLowWho },
      ].map((n) => ({ ...n, lista: visiveis.filter((m) => m.tier === n.id && m.supported) })),
    [visiveis, d]
  );

  if (!diag || !modelos) {
    return (
      <div className="stack">
        <div className="skeleton" style={{ height: 34, width: "42%" }} />
        <div className="skeleton" style={{ height: 76 }} />
        <div className="skeleton" style={{ height: 180 }} />
      </div>
    );
  }

  const comp = diag.computacao;
  const fora = visiveis.filter((m) => !m.supported);
  const semAlcance = modelos.filter((m) => !m.supported).length;
  const baixados = modelos.filter((m) => m.installed);
  const emDisco = baixados.reduce((s, m) => s + m.weights_bytes, 0);

  return (
    <>
      <header className="page__head">
        <h1>{d.models.title}</h1>
        <p>
          {provedor?.provedor === "claude_code"
            ? d.claudeElenco.lead
            : prefs.avancado
              ? d.models.advancedOn
              : d.models.lead}
        </p>
        <Porque>{d.models.why}</Porque>
      </header>

      {/* Quem executa vem antes de qual modelo: se a pessoa escolher o Claude
          Code, o catalogo abaixo deixa de valer para a proxima campanha. */}
      <Provedor
        onTrocar={(s) => {
          setProvedor(s);
          void recarregar();
        }}
      />

      {/* Com o Claude Code no comando, o catalogo do Ollama deixa de valer para
          a proxima campanha. Manter as duas listas na tela sugeriria que da
          para escolher entre as familias, e nao da: o provedor e um so. */}
      {provedor?.provedor === "claude_code" ? (
        <ElencoClaude status={provedor} />
      ) : (
      <>

      <section className="card">
        <div className="auto-grid">
          <div className="read">
            <span className="read__k">{d.boot.budgetCap}</span>
            <span className="read__v">{formatarBytes(comp.max_budget_bytes, idioma)}</span>
            <span className="read__note">{comp.mode_label}</span>
          </div>
          <div className="read">
            <span className="read__k">{d.models.fits}</span>
            <span className="read__v">
              {modelos.length - semAlcance}
              <small>/ {modelos.length}</small>
            </span>
            <span className="read__note">
              {semAlcance} {d.models.outOfReach.toLowerCase()}
            </span>
          </div>
          <div className="read">
            <span className="read__k">{d.common.installed}</span>
            <span className="read__v">{baixados.length}</span>
            <span className="read__note">
              {baixados.length ? formatarBytes(emDisco, idioma) : d.models.downloadNote}
            </span>
          </div>
        </div>
      </section>

      {/* ── controle: filtrar por familia e fixar modelo por cargo ─── */}
      <section className="card">
        <div className="filtros">
          <div className="chips">
            <button className="chip" data-on={familia === ""} onClick={() => setFamilia("")}>
              {d.models.allFamilies}
            </button>
            {familias.map((fam) => (
              <button
                key={fam}
                className="chip"
                data-on={familia === fam}
                onClick={() => setFamilia(fam)}
              >
                {fam}
              </button>
            ))}
          </div>

          <button
            className="btn btn--sm"
            data-on={prefs.avancado}
            onClick={() =>
              api
                .definirModoAvancado(!prefs.avancado)
                .then(setPrefs)
                .catch((e) => setRecado(String(e)))
            }
          >
            <IconSliders size={14} />
            {d.models.advanced}
          </button>
        </div>

        {prefs.avancado && (
          <>
            <p className="hint" style={{ margin: "16px 0 12px" }}>{d.models.advancedWhy}</p>
            <CargosAvancado
              modelos={modelos}
              prefs={prefs}
              onEscolher={(cargo, tag) =>
                api
                  .definirModeloDoCargo(cargo, tag)
                  .then(setPrefs)
                  .catch((e) => setRecado(String(e)))
              }
            />
          </>
        )}
      </section>

      {recado && (
        <div className="note" data-tone="signal" style={{ marginBottom: 18 }}>
          <span>{recado}</span>
        </div>
      )}

      {niveis.map((nivel) => (
        <section className="card" key={nivel.id}>
          <div className="card__topo">
            <h2>{nivel.titulo}</h2>
            <span className="hint">{nivel.quem}</span>
          </div>

          {nivel.lista.length === 0 ? (
            <p className="hint">{d.models.empty}</p>
          ) : (
            nivel.lista.map((m) => (
              <LinhaModelo
                key={m.tag}
                m={m}
                progresso={baixando[m.tag]}
                removendo={removendo === m.tag}
                onBaixar={baixar}
                onRemover={remover}
              />
            ))
          )}
        </section>
      ))}

      {fora.length > 0 && (
        <section className="card">
          <div className="card__topo">
            <h2>{d.models.outOfReach}</h2>
            <button className="btn btn--quiet btn--sm push" onClick={() => setVerFora((v) => !v)}>
              {verFora ? d.models.hide : f(d.models.showOut, { n: fora.length })}
            </button>
          </div>
          {verFora &&
            fora.map((m) => (
              <LinhaModelo key={m.tag} m={m} onBaixar={baixar} onRemover={remover} />
            ))}
        </section>
      )}

      </>
      )}

      <div className="row" style={{ marginTop: 28 }}>
        <span className="push" />
        <button className="btn btn--key" onClick={avancar}>
          {d.common.continue}
          <IconArrow size={16} />
        </button>
      </div>
    </>
  );
}
