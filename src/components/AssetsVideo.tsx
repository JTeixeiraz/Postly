import { useRef, useState } from "react";
import { api } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { ItemVideo, PastaAsset, ProjetoVideo } from "../types";

/** A aba de assets: onde a pessoa larga o material do vídeo.
 *
 *  TRÊS PASTAS, E A TERCEIRA É A QUE DECIDE O FLUXO. `narracao/` não é
 *  organização por gosto: é onde o modelo OLHA para saber se o vídeo tem voz.
 *  Um arquivo em `audio/` chamado "narracao-final.mp3" continua sendo trilha,
 *  porque adivinhar por nome seria adivinhação com cara de detecção.
 *
 *  Por isso a pasta de narração aparece na tela mesmo vazia, com o aviso do que
 *  o vazio significa. Uma pasta que só aparece depois de alguém procurar por
 *  ela não ensina que o lugar existe — a mesma lição da galeria de produto. */
export default function AssetsVideo({
  projeto,
  aoMudar,
}: {
  projeto: ProjetoVideo;
  aoMudar: (p: ProjetoVideo) => void;
}) {
  const { d, idioma } = useIdioma();
  const [erro, setErro] = useState<string | null>(null);

  const pastas: { id: PastaAsset; titulo: string; nota: string; itens: ItemVideo[] }[] = [
    {
      id: "imagens",
      titulo: d.videoAssets.images,
      nota: d.videoAssets.imagesNote,
      itens: projeto.imagens,
    },
    {
      id: "audio",
      titulo: d.videoAssets.audio,
      nota: d.videoAssets.audioNote,
      itens: projeto.audio,
    },
    {
      id: "narracao",
      titulo: d.videoAssets.voice,
      nota: d.videoAssets.voiceNote,
      itens: projeto.narracao,
    },
  ];

  return (
    <>
      {pastas.map((p) => (
        <Pasta
          key={p.id}
          projeto={projeto}
          pasta={p.id}
          titulo={p.titulo}
          nota={p.nota}
          itens={p.itens}
          aoMudar={aoMudar}
          aoFalhar={setErro}
        />
      ))}

      {erro && (
        <div className="note" data-tone="alert">
          <span>{erro}</span>
        </div>
      )}

      {/* O aviso sai da CONTAGEM e não de um texto fixo: ele muda de sentido
          quando a pasta enche, e um aviso que continua igual depois de
          resolvido treina a pessoa a ignorá-lo. */}
      <div className="note" data-tone={projeto.narracao.length ? "ok" : "warn"}>
        <span>
          {projeto.narracao.length
            ? d.videoAssets.hasVoice
            : d.videoAssets.noVoice}
        </span>
      </div>

      <p className="hint">
        {d.videoAssets.total} {formatarBytes(projeto.bytes, idioma)} · {projeto.caminho}
      </p>
    </>
  );
}

function Pasta({
  projeto,
  pasta,
  titulo,
  nota,
  itens,
  aoMudar,
  aoFalhar,
}: {
  projeto: ProjetoVideo;
  pasta: PastaAsset;
  titulo: string;
  nota: string;
  itens: ItemVideo[];
  aoMudar: (p: ProjetoVideo) => void;
  aoFalhar: (e: string | null) => void;
}) {
  const { d, idioma } = useIdioma();
  const input = useRef<HTMLInputElement>(null);
  const [ocupado, setOcupado] = useState(false);

  const subir = async (arquivos: FileList | null) => {
    if (!arquivos?.length) return;
    setOcupado(true);
    aoFalhar(null);
    let ultimo: ProjetoVideo | null = null;
    const falhas: string[] = [];

    // Um por vez, e o erro de um NÃO derruba os outros: quem arrastou dez
    // arquivos com um PDF no meio quer os nove que servem, com aviso sobre o
    // décimo. Mesma regra da galeria.
    for (const f of Array.from(arquivos)) {
      try {
        ultimo = await api.videoAdicionar(projeto.slug, pasta, f.name, await comoBase64(f));
      } catch (e) {
        falhas.push(`${f.name}: ${String(e)}`);
      }
    }

    if (ultimo) aoMudar(ultimo);
    if (falhas.length) aoFalhar(falhas.join(" · "));
    setOcupado(false);
    if (input.current) input.current.value = "";
  };

  const remover = async (caminho: string) => {
    try {
      aoMudar(await api.videoRemoverItem(projeto.slug, caminho));
    } catch (e) {
      aoFalhar(String(e));
    }
  };

  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{titulo}</span>
        <span className="tag">{itens.length}</span>
      </div>
      <p className="hint">{nota}</p>

      <div className="stack stack--tight">
        {itens.map((i) => (
          <div className="chave-linha" key={i.caminho}>
            <span style={{ flex: 1 }}>{i.nome}</span>
            <span className="hint num">{formatarBytes(i.bytes, idioma)}</span>
            <button
              className="btn btn--quiet btn--sm"
              onClick={() => void remover(i.caminho)}
            >
              {d.common.remove}
            </button>
          </div>
        ))}
        {!itens.length && <p className="hint">{d.videoAssets.empty}</p>}
      </div>

      <input
        ref={input}
        type="file"
        multiple
        hidden
        // O `accept` vem da pasta: quem abre o seletor na pasta de narração não
        // deve nem conseguir escolher um PNG. O Rust recusa de qualquer jeito,
        // mas recusar depois do clique é pior que não oferecer.
        accept={pasta === "imagens" ? "image/*" : "audio/*"}
        onChange={(e) => void subir(e.target.files)}
      />
      <button
        className="btn"
        disabled={ocupado}
        onClick={() => input.current?.click()}
      >
        {ocupado ? d.common.working : d.videoAssets.add}
      </button>
    </section>
  );
}

/** Arquivo → base64.
 *
 *  Sem o plugin de diálogo do Tauri o navegador não entrega o caminho real, e
 *  trazer o plugin só para isto seria peso permanente por conveniência de uma
 *  tela. Mesma decisão da galeria. */
function comoBase64(f: File): Promise<string> {
  return new Promise((ok, falha) => {
    const r = new FileReader();
    r.onload = () => ok(String(r.result));
    r.onerror = () => falha(new Error("não consegui ler o arquivo"));
    r.readAsDataURL(f);
  });
}
