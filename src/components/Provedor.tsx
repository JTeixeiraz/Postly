import { useEffect, useState } from "react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { Provedor as TipoProvedor, StatusProvedor } from "../types";

/** Quem executa os turnos: Ollama local ou o Claude Code da pessoa.
 *
 *  A escolha e de fato uma troca, nao um upgrade, e a interface diz os dois
 *  lados: o Ollama e gratis e nao manda nada para fora, o Claude Code e muito
 *  mais rapido e cobra por turno. Quem decide precisa dos dois fatos na mesma
 *  tela. */
export default function Provedor({
  onTrocar,
}: {
  /** Recebe o status novo: quem usa este componente precisa saber qual
   *  provedor ficou ativo para decidir o que desenhar abaixo. */
  onTrocar?: (s: StatusProvedor) => void;
}) {
  const { d, f } = useIdioma();
  const [status, setStatus] = useState<StatusProvedor | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  useEffect(() => {
    void api
      .statusProvedor()
      .then((s) => {
        setStatus(s);
        onTrocar?.(s);
      })
      .catch(() => {});
    // De proposito sem `onTrocar` nas dependencias: a funcao muda a cada
    // render do pai e isto viraria um laco de leitura do status.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!status) return null;

  const escolher = async (p: TipoProvedor) => {
    setErro(null);
    try {
      await api.definirProvedor(p);
      const novo = await api.statusProvedor();
      setStatus(novo);
      onTrocar?.(novo);
    } catch (e) {
      setErro(String(e));
    }
  };

  const opcoes: {
    id: TipoProvedor;
    titulo: string;
    porque: string;
    nota?: string;
    bloqueado?: boolean;
  }[] = [
    { id: "ollama", titulo: d.provider.ollama, porque: d.provider.ollamaWhy },
    {
      id: "claude_code",
      titulo: d.provider.claude,
      porque: d.provider.claudeWhy,
      nota: status.claude_disponivel
        ? f(d.provider.claudeFound, { v: status.claude_versao ?? "?" })
        : d.provider.claudeMissing,
      bloqueado: !status.claude_disponivel,
    },
  ];

  return (
    <section className="card">
      <div className="card__topo">
        <h2>{d.provider.title}</h2>
      </div>

      <div className="auto-grid">
        {opcoes.map((o) => (
          <button
            key={o.id}
            className="choice"
            data-on={status.provedor === o.id}
            aria-pressed={status.provedor === o.id}
            disabled={o.bloqueado}
            onClick={() => void escolher(o.id)}
          >
            <span className="choice__marca" aria-hidden />
            <div>
              <span className="choice__title">{o.titulo}</span>
              <div className="hint">{o.porque}</div>
              {o.nota && (
                <div className="hint" data-alerta={o.bloqueado}>
                  {o.nota}
                </div>
              )}
            </div>
          </button>
        ))}
      </div>

      {erro && (
        <div className="note" data-tone="alert">
          <span>{erro}</span>
        </div>
      )}
    </section>
  );
}
