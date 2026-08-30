import { useEffect, useState } from "react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { StatusProvedor, VagaClaude } from "../types";
import MarcaModelo from "./MarcaModelo";

const CARGO: Record<string, { pt: string; en: string }> = {
  "diretor-geral": { pt: "Diretor Geral", en: "General Director" },
  "gerente-setor": { pt: "Gerente de Setor", en: "Sector Manager" },
  "motion-designer": { pt: "Motion Designer", en: "Motion Designer" },
  criador: { pt: "Criador de Conteúdo", en: "Content Creator" },
  auditor: { pt: "Auditor", en: "Auditor" },
};

/** O elenco quando quem executa e o Claude Code.
 *
 *  Substitui o catalogo do Ollama em vez de conviver com ele: as duas listas
 *  na mesma tela sugeririam que a pessoa escolhe entre modelos das duas
 *  familias, e ela nao escolhe. O provedor e um so por vez. */
export default function ElencoClaude({ status }: { status: StatusProvedor }) {
  const { d, f, idioma } = useIdioma();
  const [vagas, setVagas] = useState<VagaClaude[] | null>(null);

  useEffect(() => {
    void api.elencoClaude().then(setVagas).catch(() => setVagas([]));
  }, []);

  if (!vagas) return <div className="skeleton" style={{ height: 140 }} />;

  return (
    <>
      <section className="card">
        <div className="card__topo">
          <span className="card__titulo">{d.claudeElenco.title}</span>
          <span className="tag" data-tone="ok">
            <span className="tag__dot" />
            {d.claudeElenco.local}
          </span>
        </div>

        <div className="elenco" style={{ ["--postas" as string]: vagas.length }}>
          {vagas.map((v) => (
            <div className="vaga" key={v.cargo}>
              <span className="vaga__marca" />
              <span className="vaga__cargo">
                {CARGO[v.cargo]?.[idioma] ?? v.cargo}
                {/* O motion nao roda em toda campanha: sem esta marca a trilha
                    promete um turno que costuma nao acontecer. */}
                {v.cargo === "motion-designer" && (
                  <em className="vaga__opcional">{d.claudeElenco.optional}</em>
                )}
              </span>
              <span className="vaga__modelo">
                <MarcaModelo familia="Anthropic" size={15} />
                {v.rotulo}
              </span>
              <span className="vaga__tag">{v.modelo}</span>
              <span className="vaga__nota">{v.porque}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="card">
        <div className="card__topo">
          <span className="card__titulo">{d.claudeElenco.howTitle}</span>
        </div>
        {/* O caminho do binario e a prova, na tela, de que isto e um processo
            local e nao uma chamada de API. Vale mostrar: e a pergunta que
            qualquer pessoa faz antes de colar uma credencial num app. */}
        <p className="hint">
          {f(d.claudeElenco.localWhy, { p: status.claude_caminho ?? "claude" })}
        </p>
        <p className="hint">{d.claudeElenco.cost}</p>

        {status.credencial_ignorada && (
          <div className="note" data-tone="warn">
            <span>{f(d.claudeElenco.envWarn, { v: status.credencial_ignorada })}</span>
          </div>
        )}
      </section>
    </>
  );
}
