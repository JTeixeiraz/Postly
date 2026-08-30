import { useCallback, useEffect, useState } from "react";
import { motion } from "motion/react";
import { api } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { CartaoModo } from "../types";

/** Quanto da máquina a campanha pode usar.
 *
 *  O modo mexe em duas alavancas ao mesmo tempo, e é por isso que os três
 *  comportamentos são de fato distintos: o teto de memória por modelo, e o
 *  piso de velocidade abaixo do qual um modelo deixa de valer a pena.
 *
 *  Os números vêm do backend calculados com a memória livre AGORA, e não de
 *  uma tabela fixa: "usa mais RAM" não diz nada a quem tem 8 GB e a quem tem
 *  64. O que diz é qual modelo cada modo traria para o cargo que decide. */
export default function ModoDesempenho() {
  const { d, f, idioma } = useIdioma();
  const [modos, setModos] = useState<CartaoModo[] | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  const ler = useCallback(() => {
    api
      .modosDeDesempenho()
      .then(setModos)
      .catch((e) => setErro(String(e)));
  }, []);

  useEffect(ler, [ler]);

  const escolher = async (slug: string) => {
    setErro(null);
    try {
      await api.definirModo(slug);
    } catch (e) {
      setErro(String(e));
    }
    // Relê sempre: a escolha muda os números dos OUTROS cartões também,
    // porque o teto de cada um é medido contra a memória livre do momento.
    ler();
  };

  if (!modos) return null;
  const maximoAtivo = modos.find((m) => m.slug === "maximo")?.ativo;

  return (
    <section className="card">
      <div className="card__topo">
        <h2>{d.desempenho.titulo}</h2>
        <span className="hint">{d.desempenho.subtitulo}</span>
      </div>

      <div className="auto-grid">
        {modos.map((m) => (
          <button
            key={m.slug}
            className="choice"
            data-on={m.ativo}
            aria-pressed={m.ativo}
            onClick={() => void escolher(m.slug)}
          >
            <span className="choice__marca" aria-hidden />
            <div>
              <span className="choice__title">{d.desempenho.nomes[m.slug]}</span>
              <div className="hint">{d.desempenho.porques[m.slug]}</div>

              {/* O que muda de concreto: o modelo que assume o cargo que
                  decide, e o teto de memória. Sem isso o seletor pediria uma
                  escolha sem mostrar a consequência dela. */}
              <div className="modo__num mono">
                <span>{m.modelo_alto}</span>
                {m.tps_alto > 0 && (
                  <span className="modo__tps">
                    {f(d.desempenho.tps, { n: m.tps_alto.toFixed(1) })}
                  </span>
                )}
              </div>
              <div className="hint mono">
                {f(d.desempenho.teto, { n: formatarBytes(m.teto_bytes, idioma) })}
              </div>
            </div>
          </button>
        ))}
      </div>

      {/* O aviso só aparece quando o modo está escolhido, não como ameaça
          prévia: assustar antes da escolha empurra para o meio sem
          argumento. */}
      {maximoAtivo && (
        <motion.p
          className="note"
          data-tone="warn"
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
        >
          {d.desempenho.avisoMaximo}
        </motion.p>
      )}

      {erro && (
        <p className="hint" data-alerta="true">
          {erro}
        </p>
      )}
    </section>
  );
}
