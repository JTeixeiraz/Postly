import { useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { api, ouvirAnalise } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { ClipeMedido, ProgressoAnalise, ProjetoVideo } from "../types";
import { useOuvinte } from "../ouvir";

/** Os vídeos que a pessoa gravou, e quanto deles é pausa.
 *
 *  ARRASTAR-E-SOLTAR, E NÃO O SELETOR DE ARQUIVO. Imagem e áudio chegam como
 *  base64 pela ponte IPC porque são pequenos; um vídeo de meio giga viraria
 *  660 MB de string na memória da janela. O evento de arrastar do Tauri entrega
 *  `paths` de verdade, e o Rust copia o arquivo — sem plugin novo e sem limite
 *  de tamanho.
 *
 *  A MEDIÇÃO APARECE ANTES DE GERAR, de propósito. "O modelo corta as pausas" é
 *  uma promessa; "13,0s de bruto viram 8,5s" é um fato que a pessoa confere
 *  antes de gastar um turno. E é o mesmo número que vai para o prompt. */
export default function ClipesVideo({
  projeto,
  aoMudar,
  clipes,
  aoMedir,
}: {
  projeto: ProjetoVideo;
  aoMudar: (p: ProjetoVideo) => void;
  clipes: ClipeMedido[] | null;
  aoMedir: (c: ClipeMedido[] | null) => void;
}) {
  const { d, f, idioma } = useIdioma();
  const [sobre, setSobre] = useState(false);
  const [ocupado, setOcupado] = useState(false);
  const [erro, setErro] = useState<string | null>(null);
  const [progresso, setProgresso] = useState<ProgressoAnalise | null>(null);

  useOuvinte(() => ouvirAnalise(setProgresso), []);

  useOuvinte(
    () =>
      getCurrentWebview().onDragDropEvent(async (e) => {
        if (e.payload.type === "enter") setSobre(true);
        else if (e.payload.type === "leave") setSobre(false);
        else if (e.payload.type === "drop") {
          setSobre(false);
          setErro(null);
          setOcupado(true);
          try {
            const [p, falhas] = await api.videoAdicionarCaminhos(
              projeto.slug,
              "clipes",
              e.payload.paths,
            );
            aoMudar(p);
            // A medição anterior deixou de valer: há material novo.
            aoMedir(null);
            if (falhas.length) setErro(falhas.join(" · "));
          } catch (x) {
            setErro(String(x));
          } finally {
            setOcupado(false);
          }
        }
      }),
    [projeto.slug, aoMudar, aoMedir],
  );

  const medir = async () => {
    setOcupado(true);
    setErro(null);
    try {
      aoMedir(await api.videoAnalisar(projeto.slug));
    } catch (e) {
      setErro(String(e));
    } finally {
      setOcupado(false);
      setProgresso(null);
    }
  };

  const remover = async (caminho: string) => {
    try {
      aoMudar(await api.videoRemoverItem(projeto.slug, caminho));
      aoMedir(null);
    } catch (e) {
      setErro(String(e));
    }
  };

  const medido = (nome: string) => clipes?.find((c) => c.nome === nome);
  const bruto = clipes?.reduce((s, c) => s + c.duracao_s, 0) ?? 0;
  const comSom =
    clipes?.reduce(
      (s, c) =>
        s + c.com_som.reduce((t, p) => t + Math.max(0, p.ate_s - p.de_s), 0),
      0,
    ) ?? 0;

  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{d.clipes.titulo}</span>
        <span className="tag">{projeto.clipes.length}</span>
      </div>
      <p className="hint">{d.clipes.nota}</p>

      <div className="solta" data-sobre={sobre}>
        {sobre ? d.clipes.solte : d.clipes.arraste}
      </div>

      {!!projeto.clipes.length && (
        <div className="stack stack--tight">
          {projeto.clipes.map((c) => {
            const m = medido(c.nome);
            return (
              <div className="chave-linha" key={c.caminho}>
                <span style={{ flex: 1 }}>{c.nome}</span>
                {m?.erro ? (
                  <span className="tag" data-tone="warn">
                    {d.clipes.erroClipe}
                  </span>
                ) : m ? (
                  <>
                    <span className="hint num">{m.duracao_s.toFixed(1)}s</span>
                    <span className="tag" data-tone={m.pausas ? "warn" : "ok"}>
                      {m.tem_audio
                        ? f(d.clipes.pausas, { n: m.pausas })
                        : d.clipes.semSom}
                    </span>
                  </>
                ) : (
                  <span className="hint num">
                    {formatarBytes(c.bytes, idioma)}
                  </span>
                )}
                <button
                  className="btn btn--quiet btn--sm"
                  onClick={() => void remover(c.caminho)}
                >
                  {d.common.remove}
                </button>
              </div>
            );
          })}
        </div>
      )}

      {!!projeto.clipes.length && (
        <div className="row">
          <button
            className="btn"
            disabled={ocupado}
            onClick={() => void medir()}
          >
            {ocupado
              ? d.clipes.analisando
              : clipes
                ? d.clipes.remedir
                : d.clipes.medir}
          </button>
          {/* O ganho em números. É o mesmo dado que vai para o prompt, então a
              tela não está prometendo nada que o modelo não vá receber. */}
          {clipes && bruto > 0 && (
            <span className="hint">
              {bruto.toFixed(1)}s {d.clipes.bruto} → {comSom.toFixed(1)}s{" "}
              {d.clipes.falado}
              {" · "}
              <strong>
                {f(d.clipes.ganho, { n: (bruto - comSom).toFixed(1) })}
              </strong>
            </span>
          )}
        </div>
      )}

      {progresso && ocupado && (
        <div className="provisao__barra">
          <span style={{ width: `${Math.round(progresso.percent * 100)}%` }} />
        </div>
      )}

      {clipes && <p className="hint">{d.clipes.porQue}</p>}

      {erro && (
        <div className="note" data-tone="alert">
          <span>{erro}</span>
        </div>
      )}
    </section>
  );
}
