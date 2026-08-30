import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { Vaga } from "../types";

/** Tokens que um turno costuma gerar, por cargo.
 *
 *  Numeros de observacao, nao de especificacao: quem decide escreve briefing
 *  longo, quem devolve JSON escreve pouco. Servem para a ordem de grandeza da
 *  estimativa, que e o que importa aqui: distinguir "cinco minutos" de "uma
 *  hora" muda a decisao da pessoa; errar por 20% nao muda nada. */
const TOKENS_POR_CARGO: Record<string, number> = {
  "diretor-geral": 700,
  "gerente-setor": 900,
  criador: 600,
  auditor: 450,
};

function humano(segundos: number, idioma: "pt" | "en"): string {
  if (segundos < 90) {
    return idioma === "pt" ? `${Math.round(segundos)} s` : `${Math.round(segundos)}s`;
  }
  const min = Math.round(segundos / 60);
  if (min < 90) return `${min} min`;
  const h = Math.floor(min / 60);
  const resto = min % 60;
  return resto ? `${h} h ${resto} min` : `${h} h`;
}

/** O que acontece ao apertar o botao, antes de apertar.
 *
 *  A campanha leva minutos ou horas nesta maquina, e ate agora a unica forma de
 *  descobrir isso era comecando. Contar os turnos e estimar o tempo pela vazao
 *  medida transforma a decisao de rodar numa decisao informada. */
export default function PlanoExecucao({
  redes,
  rodadas,
}: {
  redes: string[];
  rodadas: number;
}) {
  const { d, idioma } = useIdioma();
  const [elenco, setElenco] = useState<Vaga[] | null>(null);

  useEffect(() => {
    void api.elenco().then(setElenco).catch(() => setElenco([]));
  }, []);

  const plano = useMemo(() => {
    if (redes.length === 0) return null;
    const multi = redes.length > 1;

    // O mesmo percurso que o orquestrador executa: diretor so com mais de uma
    // rede, um gerente por rede, um criador para todas, auditor, e o decisor.
    const percurso: string[] = [];
    if (multi) percurso.push("diretor-geral");
    redes.forEach(() => percurso.push("gerente-setor"));
    percurso.push("criador", "auditor", multi ? "diretor-geral" : "gerente-setor");

    // Gerente e diretor decidem uma vez; o ciclo que repete e criar, auditar e
    // decidir. Por isso a rodada extra nao multiplica o percurso inteiro.
    const porRodada = ["criador", "auditor", multi ? "diretor-geral" : "gerente-setor"];
    const todos = [...percurso, ...Array.from({ length: rodadas - 1 }, () => porRodada).flat()];

    const vazao = (cargo: string) =>
      elenco?.find((v) => v.cargo === cargo)?.estimated_tps ?? 0;

    const segundos = todos.reduce((soma, cargo) => {
      const tps = vazao(cargo);
      if (tps <= 0) return soma;
      return soma + (TOKENS_POR_CARGO[cargo] ?? 600) / tps;
    }, 0);

    return {
      turnos: todos.length,
      imagens: redes.length * rodadas,
      segundos,
      // Sem elenco resolvido ainda, nao ha vazao para estimar tempo.
      temTempo: segundos > 0,
    };
  }, [redes, rodadas, elenco]);

  if (!plano) {
    return (
      <section className="card">
        <div className="card__topo">
          <span className="card__titulo">{d.campaign.plan}</span>
        </div>
        <p className="hint">{d.campaign.planPick}</p>
      </section>
    );
  }

  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{d.campaign.plan}</span>
      </div>

      <div className="plano">
        <div className="plano__num">
          <span className="plano__valor num">{plano.turnos}</span>
          <span className="plano__rot">{d.campaign.planTurns}</span>
        </div>
        <div className="plano__num">
          <span className="plano__valor num">{plano.imagens}</span>
          <span className="plano__rot">{d.campaign.planImages}</span>
        </div>
        {plano.temTempo && (
          <div className="plano__num">
            <span className="plano__valor num">{humano(plano.segundos, idioma)}</span>
            <span className="plano__rot">{d.campaign.planTime}</span>
          </div>
        )}
      </div>

      {plano.temTempo && <p className="hint">{d.campaign.planEstimate}</p>}
    </section>
  );
}
