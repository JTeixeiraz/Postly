import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { ItemGaleria, PastaGaleria } from "../types";
import { IconDownload, IconOpen, IconTrash } from "./Icons";

/** Pastas de produto: os assets de uma marca, guardados para reuso.
 *
 *  Antes disto, cada campanha recomeçava do zero — a pessoa subia foto por
 *  foto, toda vez, para o mesmo produto.
 *
 *  A estrutura de pastas espelha a distinção que o produto já faz: o que está
 *  na pasta pode aparecer na peça; o que está em `referencias/` é só direção
 *  de estilo. Não é organização por gosto — mandar a arte de outra marca para
 *  o modelo copiar é o caminho mais curto para sair logotipo alheio na peça. */
export default function Galeria() {
  const { d, f, idioma } = useIdioma();
  const [pastas, setPastas] = useState<PastaGaleria[] | null>(null);
  const [aberta, setAberta] = useState<string | null>(null);
  const [nome, setNome] = useState("");
  const [erro, setErro] = useState<string | null>(null);
  const entrada = useRef<HTMLInputElement>(null);
  const alvo = useRef<{ slug: string; refs: boolean } | null>(null);

  const ler = useCallback(() => {
    api.galeriaListar().then(setPastas).catch((e) => setErro(String(e)));
  }, []);
  useEffect(ler, [ler]);

  const criar = async () => {
    if (!nome.trim()) return;
    setErro(null);
    try {
      const p = await api.galeriaCriar(nome);
      setNome("");
      setAberta(p.slug);
      ler();
    } catch (e) {
      setErro(String(e));
    }
  };

  const pedirArquivos = (slug: string, refs: boolean) => {
    alvo.current = { slug, refs };
    entrada.current?.click();
  };

  const receber = async (lista: FileList | null) => {
    const destino = alvo.current;
    if (!lista?.length || !destino) return;
    setErro(null);
    // O backend recebe base64 porque, sem o plugin de diálogo do Tauri, o
    // navegador não entrega o caminho real do arquivo.
    const arquivos = await Promise.all(
      [...lista].map(
        (f) =>
          new Promise<{ nome: string; dados: string }>((ok, falha) => {
            const leitor = new FileReader();
            leitor.onload = () => ok({ nome: f.name, dados: String(leitor.result) });
            leitor.onerror = () => falha(new Error(f.name));
            leitor.readAsDataURL(f);
          })
      )
    ).catch(() => null);
    if (!arquivos) return setErro(d.galeria.erroLeitura);
    try {
      await api.galeriaAdicionar(destino.slug, arquivos, destino.refs);
    } catch (e) {
      setErro(String(e));
    }
    if (entrada.current) entrada.current.value = "";
    ler();
  };

  const apagarItem = async (caminho: string) => {
    await api.galeriaRemoverItem(caminho).catch((e) => setErro(String(e)));
    ler();
  };

  const apagarPasta = async (slug: string) => {
    await api.galeriaRemoverPasta(slug).catch((e) => setErro(String(e)));
    if (aberta === slug) setAberta(null);
    ler();
  };

  if (!pastas) return null;

  const grade = (itens: ItemGaleria[], vazio: string) =>
    itens.length === 0 ? (
      <p className="hint">{vazio}</p>
    ) : (
      <div className="galeria__grade">
        {itens.map((i) => (
          <figure className="asset" key={i.caminho}>
            <img src={convertFileSrc(i.caminho)} alt={i.nome} loading="lazy" />
            <figcaption>
              <span className="asset__nome">{i.nome}</span>
              <button
                className="btn btn--quiet btn--sm"
                title={d.galeria.apagar}
                onClick={() => void apagarItem(i.caminho)}
              >
                <IconTrash size={13} />
              </button>
            </figcaption>
          </figure>
        ))}
      </div>
    );

  return (
    <section className="card">
      <div className="card__topo">
        <h2>{d.galeria.titulo}</h2>
        <span className="hint">{d.galeria.subtitulo}</span>
      </div>

      <input
        ref={entrada}
        type="file"
        accept="image/png,image/jpeg,image/webp"
        multiple
        hidden
        onChange={(e) => void receber(e.target.files)}
      />

      <div className="row row--tight">
        <input
          className="campo"
          placeholder={d.galeria.novaPasta}
          value={nome}
          onChange={(e) => setNome(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && void criar()}
        />
        <button className="btn btn--key" onClick={() => void criar()} disabled={!nome.trim()}>
          {d.galeria.criar}
        </button>
      </div>

      {pastas.length === 0 && <p className="hint">{d.galeria.nenhuma}</p>}

      {pastas.map((p) => (
        <div className="pasta" key={p.slug} data-on={aberta === p.slug}>
          <div className="row row--tight">
            <button
              className="btn btn--quiet btn--sm"
              onClick={() => setAberta(aberta === p.slug ? null : p.slug)}
            >
              {p.nome}
            </button>
            <span className="hint mono">
              {f(d.galeria.resumo, {
                n: p.itens.length,
                r: p.referencias.length,
                b: formatarBytes(p.bytes, idioma),
              })}
            </span>
            <span className="push" />
            <button
              className="btn btn--quiet btn--sm"
              title={d.galeria.abrirPasta}
              onClick={() => void api.abrirNoSistema(p.caminho)}
            >
              <IconOpen size={13} />
            </button>
            <button
              className="btn btn--quiet btn--sm"
              title={d.galeria.apagarPasta}
              onClick={() => void apagarPasta(p.slug)}
            >
              <IconTrash size={13} />
            </button>
          </div>

          <AnimatePresence>
            {aberta === p.slug && (
              <motion.div
                className="stack"
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
              >
                <div className="row row--tight">
                  <strong style={{ color: "var(--ink)" }}>{d.galeria.doProduto}</strong>
                  <span className="push" />
                  <button className="btn btn--sm" onClick={() => pedirArquivos(p.slug, false)}>
                    <IconDownload size={13} />
                    {d.galeria.adicionar}
                  </button>
                </div>
                <p className="hint">{d.galeria.doProdutoPorque}</p>
                {grade(p.itens, d.galeria.semItens)}

                <div className="row row--tight">
                  <strong style={{ color: "var(--ink)" }}>{d.galeria.deTerceiros}</strong>
                  <span className="push" />
                  <button className="btn btn--sm" onClick={() => pedirArquivos(p.slug, true)}>
                    <IconDownload size={13} />
                    {d.galeria.adicionar}
                  </button>
                </div>
                <p className="hint">{d.galeria.deTerceirosPorque}</p>
                {grade(p.referencias, d.galeria.semRefs)}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      ))}

      {erro && (
        <p className="hint" data-alerta="true">
          {erro}
        </p>
      )}
    </section>
  );
}
