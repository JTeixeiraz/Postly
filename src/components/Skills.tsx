import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import { PREFS_VAZIAS, type Preferencias, type Skill } from "../types";
import { IconSliders, IconTrash } from "./Icons";
import Selecao from "./Selecao";

const CARGOS = [
  { slug: "diretor-geral", chave: "diretor_geral" },
  { slug: "gerente-setor", chave: "gerente_setor" },
  { slug: "criador", chave: "criador" },
  { slug: "motion-designer", chave: "motion_designer" },
  { slug: "auditor", chave: "auditor" },
] as const;

/** A skill que ja vive dentro do produto.
 *
 *  A doutrina de marketing (frameworks por canal, as regras duras, a lista de
 *  proibicoes) e compilada nos prompts de todos os cargos que produzem: ela nao
 *  e opcional e nao pode ser desligada. Aparece aqui porque a aba de skills e
 *  onde a pessoa vem descobrir o que os modelos estao recebendo, e omitir a
 *  maior instrucao de todas faria a lista mentir. */
const EMBUTIDA = {
  nome: "marketing-head",
  cargo: "",
} as const;

/** Acima disto a instrucao come o orcamento de contexto do cargo. */
const TETO_AVISO = 1200;

/** Skills: instrucoes que a pessoa acrescenta ao prompt de um cargo.
 *
 *  Fica escondido de proposito, ao lado do guia. Isto nao e configuracao de
 *  uso diario: e a alavanca que mais facilmente estraga o resultado, e o
 *  estrago e silencioso — as pecas ficam piores sem nada dar erro. Por isso a
 *  skill nasce desligada e o aviso e a primeira coisa do painel. */
export default function Skills() {
  const { d } = useIdioma();
  const [aberto, setAberto] = useState(false);
  const [prefs, setPrefs] = useState<Preferencias>(PREFS_VAZIAS);
  const [rascunho, setRascunho] = useState<Skill | null>(null);
  const [erro, setErro] = useState<string | null>(null);
  const caixa = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (aberto) void api.preferencias().then(setPrefs).catch(() => {});
  }, [aberto]);

  useEffect(() => {
    if (!aberto) return;
    const fora = (e: MouseEvent) => {
      if (!caixa.current?.contains(e.target as Node)) setAberto(false);
    };
    document.addEventListener("mousedown", fora);
    return () => document.removeEventListener("mousedown", fora);
  }, [aberto]);

  const salvar = async (s: Skill) => {
    setErro(null);
    try {
      setPrefs(await api.salvarSkill(s.id, s.nome, s.texto, s.cargo, s.ativa));
      setRascunho(null);
    } catch (e) {
      setErro(String(e));
    }
  };

  const ativas = prefs.skills.filter((s) => s.ativa).length;

  return (
    <div className="skills" ref={caixa}>
      <button
        className="btn btn--quiet btn--sm"
        onClick={() => setAberto((a) => !a)}
        aria-expanded={aberto}
      >
        <IconSliders size={15} />
        <span>{d.skills.open}</span>
        {ativas > 0 && <span className="skills__conta">{ativas}</span>}
      </button>

      <AnimatePresence>
        {aberto && (
          <motion.div
            className="skills__painel card"
            initial={{ opacity: 0, y: -6, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.99 }}
            transition={{ duration: 0.16, ease: [0.16, 1, 0.3, 1] }}
          >
            <div className="card__topo">
              <h2>{d.skills.title}</h2>
              <button className="btn btn--quiet btn--sm push" onClick={() => setAberto(false)}>
                {d.common.close}
              </button>
            </div>
            <p className="hint">{d.skills.lead}</p>

            {/* O aviso vem antes da lista, nao depois: quem chega aqui precisa
                ler o risco antes de ver o botao de adicionar. */}
            <div className="note" data-tone="warn">
              <strong>{d.skills.warn}</strong>
              <span>{d.skills.warnBody}</span>
            </div>

            {erro && (
              <div className="note" data-tone="alert">
                <span>{erro}</span>
              </div>
            )}

            <div className="skills__lista">
              {/* Embutida: sempre ativa, sem interruptor e sem lixeira. Mostrar
                  controles que nao funcionam seria pior que nao mostrar nada. */}
              <div className="skill skill--embutida">
                <div className="skill__topo">
                  <span className="skill__nome">{EMBUTIDA.nome}</span>
                  <span className="tag" data-tone="ok">
                    <span className="tag__dot" />
                    {d.skills.builtin}
                  </span>
                  <span className="tag push">{d.skills.allRoles}</span>
                </div>
                <p className="skill__texto">{d.skills.builtinBody}</p>
              </div>

              {prefs.skills.length === 0 && !rascunho && (
                <p className="hint">{d.skills.empty}</p>
              )}

              {prefs.skills.map((s) => (
                <div className="skill" key={s.id}>
                  <div className="skill__topo">
                    <label className="skill__on">
                      <input
                        type="checkbox"
                        checked={s.ativa}
                        onChange={(e) => void salvar({ ...s, ativa: e.target.checked })}
                      />
                      <span className="skill__nome">{s.nome}</span>
                    </label>
                    <span className="tag">
                      {s.cargo
                        ? d.roles[CARGOS.find((c) => c.slug === s.cargo)?.chave ?? "criador"]
                        : d.skills.allRoles}
                    </span>
                    <button
                      className="btn btn--quiet btn--sm push"
                      onClick={() => api.removerSkill(s.id).then(setPrefs)}
                      title={d.models.remove}
                    >
                      <IconTrash size={13} />
                    </button>
                  </div>
                  <p className="skill__texto">{s.texto}</p>
                </div>
              ))}
            </div>

            {rascunho ? (
              <Editor
                skill={rascunho}
                onMudar={setRascunho}
                onSalvar={() => void salvar(rascunho)}
                onCancelar={() => setRascunho(null)}
              />
            ) : (
              <button
                className="btn btn--sm"
                onClick={() =>
                  setRascunho({ id: "", nome: "", texto: "", cargo: "", ativa: false })
                }
              >
                {d.skills.add}
              </button>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function Editor({
  skill,
  onMudar,
  onSalvar,
  onCancelar,
}: {
  skill: Skill;
  onMudar: (s: Skill) => void;
  onSalvar: () => void;
  onCancelar: () => void;
}) {
  const { d, f } = useIdioma();
  const longa = skill.texto.length > TETO_AVISO;

  return (
    <div className="skill-editor">
      <label className="field">
        <span>{d.skills.name}</span>
        <input
          type="text"
          value={skill.nome}
          placeholder={d.skills.namePlaceholder}
          onChange={(e) => onMudar({ ...skill, nome: e.target.value })}
        />
      </label>

      <label className="field">
        <span>{d.skills.role}</span>
        <Selecao
          valor={skill.cargo}
          onEscolher={(v) => onMudar({ ...skill, cargo: v })}
          opcoes={[
            { valor: "", rotulo: d.skills.allRoles },
            ...CARGOS.map((c) => ({ valor: c.slug, rotulo: d.roles[c.chave] })),
          ]}
        />
      </label>

      <label className="field">
        <span>{d.skills.text}</span>
        <textarea
          value={skill.texto}
          placeholder={d.skills.textPlaceholder}
          onChange={(e) => onMudar({ ...skill, texto: e.target.value })}
        />
        <span className="hint" data-alerta={longa}>
          {f(d.skills.chars, { n: skill.texto.length })}
          {longa && ` · ${d.skills.tooLong}`}
        </span>
      </label>

      <div className="row">
        <label className="row row--tight">
          <input
            type="checkbox"
            checked={skill.ativa}
            onChange={(e) => onMudar({ ...skill, ativa: e.target.checked })}
          />
          <span className="hint">{d.skills.active}</span>
        </label>
        <span className="push" />
        <button className="btn btn--quiet btn--sm" onClick={onCancelar}>
          {d.common.close}
        </button>
        <button
          className="btn btn--key btn--sm"
          onClick={onSalvar}
          disabled={!skill.nome.trim() || !skill.texto.trim()}
        >
          {d.skills.save}
        </button>
      </div>
    </div>
  );
}
