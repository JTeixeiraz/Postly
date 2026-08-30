import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api, ouvirAtualizacao } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { Atualizacao, ProgressoBaixa } from "../types";
import { IconArrow, IconSpinner } from "./Icons";

const ADIADA = "postly:atualizacao-adiada";

/** Oferece a versão nova, e instala se a pessoa aceitar.
 *
 *  A verificação acontece na abertura porque é o único momento em que
 *  interromper não custa nada: no meio de uma campanha, um modal roubaria a
 *  tela de quem está esperando um turno terminar.
 *
 *  Adiar guarda a versão recusada, não um prazo. Perguntar de novo pela mesma
 *  versão a cada abertura é o caminho mais curto para a pessoa aprender a
 *  fechar o modal sem ler — e aí ela também não lê o que importa. */
export default function ModalAtualizacao() {
  const { d, f, idioma } = useIdioma();
  const [nova, setNova] = useState<Atualizacao | null>(null);
  const [baixando, setBaixando] = useState<ProgressoBaixa | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const [pronta, setPronta] = useState(false);

  useEffect(() => {
    // Falha em silêncio de propósito: estar sem rede não é assunto de quem
    // abriu o aplicativo para escrever uma campanha.
    void api
      .verificarAtualizacao()
      .then((r) => {
        if (r.disponivel && localStorage.getItem(ADIADA) !== r.versao_nova) {
          setNova(r);
        }
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    let parar: (() => void) | undefined;
    void ouvirAtualizacao(setBaixando).then((x) => (parar = x));
    return () => parar?.();
  }, []);

  const instalar = async () => {
    if (!nova?.url_instalador) return;
    setErro(null);
    setBaixando({ baixado: 0, total: nova.tamanho_bytes ?? 0, percent: 0 });
    try {
      await api.instalarAtualizacao(nova.url_instalador);
      setPronta(true);
    } catch (e) {
      setErro(String(e));
    } finally {
      setBaixando(null);
    }
  };

  const adiar = () => {
    if (nova?.versao_nova) localStorage.setItem(ADIADA, nova.versao_nova);
    setNova(null);
  };

  const emAndamento = baixando !== null;

  return (
    <AnimatePresence>
      {nova && (
        <motion.div
          className="veu"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          // Enquanto baixa, clicar fora não fecha: interromper no meio deixaria
          // um arquivo pela metade em disco e nenhuma explicação na tela.
          onClick={() => !emAndamento && !pronta && adiar()}
        >
          <motion.div
            className="modal modal--atualizacao"
            role="dialog"
            aria-modal="true"
            aria-labelledby="atu-titulo"
            initial={{ opacity: 0, y: 16, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 10, scale: 0.99 }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal__topo">
              <span className="modal__selo" data-tone="live">
                {emAndamento ? <IconSpinner size={15} /> : <IconArrow size={15} />}
              </span>
              <div>
                <h2 id="atu-titulo">
                  {pronta ? d.atualizacao.pronta : emAndamento ? d.atualizacao.baixando : d.atualizacao.titulo}
                </h2>
                <span className="hint">
                  {f(d.atualizacao.dePara, {
                    atual: nova.versao_atual,
                    nova: nova.versao_nova ?? "?",
                  })}
                </span>
              </div>
            </div>

            {/* Andamento: a barra e o número. Um download de dezenas de
                megabytes sem sinal na tela lê como travamento. */}
            {emAndamento && (
              <div className="provisao">
                {/* `width`, e não `scaleX`: a barra do Preparo já usa largura,
                    e a escala partia do centro — o preenchimento crescia para
                    os dois lados em vez de avançar da esquerda. */}
                <div className="provisao__barra">
                  <span style={{ width: `${Math.max(baixando.percent, 1)}%` }} />
                </div>
                <div className="row row--tight">
                  <span className="hint num">{baixando.percent}%</span>
                  {baixando.total > 0 && (
                    <span className="hint num">
                      {formatarBytes(baixando.baixado, idioma)} / {formatarBytes(baixando.total, idioma)}
                    </span>
                  )}
                </div>
              </div>
            )}

            {pronta && (
              <div className="note" data-tone="signal">
                <span>{d.atualizacao.instaladorAberto}</span>
              </div>
            )}

            {!emAndamento && !pronta && (
              <>
                {nova.notas && (
                  <div className="field">
                    <span>{d.atualizacao.oQueMudou}</span>
                    <pre className="raw modal__detalhe">{nova.notas}</pre>
                  </div>
                )}
                <span className="hint">
                  {nova.tamanho_bytes
                    ? f(d.atualizacao.tamanho, { t: formatarBytes(nova.tamanho_bytes, idioma) })
                    : d.atualizacao.semTamanho}
                </span>
              </>
            )}

            {erro && (
              <div className="note" data-tone="alert">
                <span>{erro}</span>
              </div>
            )}

            <div className="row">
              {!emAndamento && !pronta && (
                <>
                  <button className="btn btn--key btn--sm" onClick={instalar}>
                    {d.atualizacao.instalar}
                  </button>
                  <button className="btn btn--quiet btn--sm" onClick={adiar}>
                    {d.atualizacao.agoraNao}
                  </button>
                </>
              )}
              {pronta && (
                <button className="btn btn--key btn--sm" onClick={() => setNova(null)}>
                  {d.common.close}
                </button>
              )}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
