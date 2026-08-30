import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { CartaoImagem } from "../types";
import MarcaImagem from "./MarcaImagem";
import { IconCheck, IconOpen } from "./Icons";

/** Quem gera a arte da peça.
 *
 *  A escolha é por logo e não por lista: são cinco serviços que a pessoa já
 *  reconhece de vista, e um seletor de texto obrigaria a ler cinco nomes para
 *  achar o que ela já sabe qual é. O cartão do serviço ativo é o único sólido.
 *
 *  Cada serviço carrega o selo do que foi testado. Só o Gemini rodou contra a
 *  API real; os outros quatro foram escritos a partir da documentação oficial.
 *  Esconder isso faria a primeira falha parecer culpa de quem colou a chave. */
export default function EscolhaImagem() {
  const { d, f } = useIdioma();
  const [cartoes, setCartoes] = useState<CartaoImagem[] | null>(null);
  const [editando, setEditando] = useState<string | null>(null);
  const [rascunho, setRascunho] = useState("");
  const [testando, setTestando] = useState<string | null>(null);
  const [recado, setRecado] = useState<{ slug: string; texto: string; erro: boolean } | null>(null);

  const recarregar = useCallback(async () => {
    setCartoes(await api.provedoresDeImagem().catch(() => []));
  }, []);

  useEffect(() => {
    void recarregar();
  }, [recarregar]);

  if (!cartoes) return <div className="skeleton" style={{ height: 128 }} />;

  const escolher = async (slug: string) => {
    setRecado(null);
    setCartoes(await api.definirProvedorImagem(slug).catch(() => cartoes));
  };

  const salvar = async (slug: string) => {
    setRecado(null);
    try {
      setCartoes(await api.salvarChaveDeImagem(slug, rascunho));
      setEditando(null);
      setRascunho("");
    } catch (e) {
      setRecado({ slug, texto: String(e), erro: true });
    }
  };

  const testar = async (slug: string) => {
    setTestando(slug);
    setRecado(null);
    try {
      const r = await api.testarProvedorImagem(slug);
      setRecado({ slug, texto: f(d.imgProvider.testOk, { r }), erro: false });
    } catch (e) {
      setRecado({ slug, texto: String(e), erro: true });
    } finally {
      setTestando(null);
    }
  };

  return (
    <section className="card">
      <div className="card__topo">
        <h2>{d.imgProvider.title}</h2>
        <span className="card__nota">{d.imgProvider.lead}</span>
      </div>

      <div className="provedores">
        {cartoes.map((c) => (
          <button
            key={c.slug}
            className="provedor"
            data-on={c.ativo}
            aria-pressed={c.ativo}
            onClick={() => void escolher(c.slug)}
          >
            {c.ativo && (
              // Uma camada só, compartilhada: ela desliza de um cartão para o
              // outro em vez de sumir aqui e aparecer ali.
              <motion.span className="provedor__bolha" layoutId="provedor-ativo" transition={{ type: "spring", stiffness: 380, damping: 32 }} />
            )}
            <span className="provedor__conteudo">
              <MarcaImagem slug={c.slug} size={27} />
              <span className="provedor__nome">{c.label}</span>
              <span className="provedor__estado">
                {c.tem_chave ? (
                  <>
                    <IconCheck size={11} />
                    {c.dica}
                  </>
                ) : (
                  d.imgProvider.noKey
                )}
              </span>
            </span>
          </button>
        ))}
      </div>

      {cartoes.map((c) =>
        c.ativo ? (
          <motion.div
            key={c.slug}
            className="card__corpo stack"
            initial={{ opacity: 0, y: -6 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <div className="row row--tight">
              <strong style={{ color: "var(--ink)" }}>{c.label}</strong>
              {c.verificado ? (
                <span className="tag" data-tone="ok">
                  <span className="tag__dot" />
                  {d.imgProvider.verified}
                </span>
              ) : (
                <span className="tag" data-tone="warn">
                  <span className="tag__dot" />
                  {d.imgProvider.untested}
                </span>
              )}
              <a
                className="btn btn--quiet btn--sm push"
                href={c.url_da_chave}
                target="_blank"
                rel="noreferrer"
              >
                {d.imgProvider.getKey}
                <IconOpen size={13} />
              </a>
            </div>

            {!c.verificado && <p className="hint">{d.imgProvider.untestedWhy}</p>}

            {editando === c.slug ? (
              <div className="row">
                <input
                  type="password"
                  autoFocus
                  value={rascunho}
                  placeholder={c.precisa_de_par ? "id:segredo" : d.imgProvider.keyPlaceholder}
                  onChange={(e) => setRascunho(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && void salvar(c.slug)}
                />
                <button className="btn btn--key btn--sm" onClick={() => void salvar(c.slug)}>
                  {d.common.save}
                </button>
                <button
                  className="btn btn--quiet btn--sm"
                  onClick={() => {
                    setEditando(null);
                    setRascunho("");
                  }}
                >
                  {d.common.close}
                </button>
              </div>
            ) : (
              <div className="row">
                <button className="btn btn--sm" onClick={() => setEditando(c.slug)}>
                  {c.tem_chave ? d.imgProvider.replaceKey : d.imgProvider.addKey}
                </button>
                {c.tem_chave && (
                  <button
                    className="btn btn--sm"
                    disabled={testando !== null}
                    onClick={() => void testar(c.slug)}
                  >
                    {testando === c.slug ? d.imgProvider.testing : d.imgProvider.test}
                  </button>
                )}
              </div>
            )}

            {c.precisa_de_par && <p className="hint">{d.imgProvider.pairHint}</p>}

            <AnimatePresence>
              {recado?.slug === c.slug && (
                <motion.div
                  className="note"
                  data-tone={recado.erro ? "alert" : "ok"}
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: "auto" }}
                  exit={{ opacity: 0, height: 0 }}
                >
                  <span>{recado.texto}</span>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        ) : null
      )}
    </section>
  );
}
