import { useEffect, useState } from "react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { Provedor, StatusProvedor, VagaClaude } from "../types";
import MarcaModelo from "./MarcaModelo";

const CARGO: Record<string, { pt: string; en: string }> = {
  "diretor-geral": { pt: "Diretor Geral", en: "General Director" },
  "gerente-setor": { pt: "Gerente de Setor", en: "Sector Manager" },
  "motion-designer": { pt: "Motion Designer", en: "Motion Designer" },
  criador: { pt: "Criador de Conteúdo", en: "Content Creator" },
  auditor: { pt: "Auditor", en: "Auditor" },
};

/** O elenco quando quem executa e um CLI de fora — Claude Code ou Gemini CLI.
 *
 *  Substitui o catalogo do Ollama em vez de conviver com ele: as duas listas
 *  na mesma tela sugeririam que a pessoa escolhe entre modelos das duas
 *  familias, e ela nao escolhe. O provedor e um so por vez.
 *
 *  Um componente para os dois provedores, e nao um por provedor: o que muda
 *  entre eles e a familia do glifo e de onde vem a lista. Duplicar deixaria
 *  duas telas que precisam ser corrigidas juntas para sempre, e a segunda
 *  seria esquecida. */
export default function ElencoExterno({
  status,
  provedor,
}: {
  status: StatusProvedor;
  provedor: Extract<Provedor, "claude_code" | "antigravity">;
}) {
  const { d, f, idioma } = useIdioma();
  const [vagas, setVagas] = useState<VagaClaude[] | null>(null);

  const agy = provedor === "antigravity";

  useEffect(() => {
    setVagas(null);
    const carregar = agy ? api.elencoAntigravity() : api.elencoClaude();
    void carregar.then(setVagas).catch(() => setVagas([]));
  }, [agy]);

  if (!vagas) return <div className="skeleton" style={{ height: 140 }} />;

  const caminho =
    (agy ? status.agy_caminho : status.claude_caminho) ??
    (agy ? "agy" : "claude");
  // A do Claude Code é removida do processo filho; a do Antigravity não, e o
  // texto do aviso diz coisas diferentes por isso. Ver `antigravity/ambiente.rs`.
  const credencial = agy
    ? status.agy_credencial_no_ambiente
    : status.credencial_ignorada;

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

        <div
          className="elenco"
          style={{ ["--postas" as string]: vagas.length }}
        >
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
                <MarcaModelo familia={agy ? "Google" : "Anthropic"} size={15} />
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
        <p className="hint">{f(d.claudeElenco.localWhy, { p: caminho })}</p>
        <p className="hint">{agy ? d.agyElenco.cost : d.claudeElenco.cost}</p>

        {credencial && (
          <div className="note" data-tone="warn">
            <span>
              {f(agy ? d.agyElenco.envWarn : d.claudeElenco.envWarn, {
                v: credencial,
              })}
            </span>
          </div>
        )}
      </section>
    </>
  );
}
