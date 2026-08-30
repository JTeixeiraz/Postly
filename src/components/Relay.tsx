import { useIdioma } from "../i18n";
import type { EventoEstagio } from "../types";

export type EstadoPosta = "espera" | "ativo" | "feito" | "falhou";

export interface Posta {
  passo: number;
  cargo: keyof ReturnType<typeof useIdioma>["d"]["roles"];
  rede?: string | null;
  modelo?: string | null;
  estado: EstadoPosta;
  detalhe?: string;
}

/** O percurso planejado, antes de qualquer modelo subir.
 *
 *  Mostrar isto na tela de configuração é o que transforma "aperte o botão e
 *  espere" em "veja quem vai trabalhar e em que ordem". */
export function planejar(redes: string[], rodadas: number): Posta[] {
  const multi = redes.length > 1;
  const postas: Posta[] = [];
  let passo = 0;

  if (multi) {
    postas.push({ passo: ++passo, cargo: "diretor_geral", estado: "espera" });
  }
  // Antes de escolher rede a trilha ainda precisa fazer sentido: mostra a forma
  // minima do percurso, com um gerente sem setor definido. Sem isto o desenho
  // comeca no Criador, sugerindo um fluxo que o sistema nunca executa.
  const setores: (string | null)[] = redes.length > 0 ? redes : [null];
  for (const rede of setores) {
    postas.push({ passo: ++passo, cargo: "gerente_setor", rede, estado: "espera" });
  }
  // Um criador só, para todas as redes.
  postas.push({ passo: ++passo, cargo: "criador", estado: "espera" });
  postas.push({ passo: ++passo, cargo: "auditor", estado: "espera" });
  postas.push({
    passo: ++passo,
    cargo: multi ? "diretor_geral" : "gerente_setor",
    rede: multi ? null : (redes[0] ?? null),
    estado: "espera",
    detalhe: rodadas > 1 ? `até ${rodadas}×` : undefined,
  });
  return postas;
}

/** Traduz os eventos que o Rust emite em postas com estado. */
export function postasDeEventos(eventos: EventoEstagio[]): Posta[] {
  const porPasso = new Map<number, EventoEstagio[]>();
  for (const e of eventos) {
    porPasso.set(e.step, [...(porPasso.get(e.step) ?? []), e]);
  }
  return [...porPasso.entries()]
    .sort(([a], [b]) => a - b)
    .map(([passo, lista]) => {
      const ultimo = lista[lista.length - 1];
      const estado: EstadoPosta =
        ultimo.stage === "concluido" ? "feito" : ultimo.stage === "falhou" ? "falhou" : "ativo";
      return {
        passo,
        cargo: cargoDoRotulo(ultimo.role),
        rede: ultimo.network,
        modelo: ultimo.model,
        estado,
        detalhe: ultimo.detail,
      };
    });
}

/** O Rust manda o rótulo já legível; aqui voltamos à chave para traduzir. */
function cargoDoRotulo(rotulo: string): Posta["cargo"] {
  const n = rotulo.toLowerCase();
  if (n.includes("diretor")) return "diretor_geral";
  if (n.includes("gerente")) return "gerente_setor";
  if (n.includes("criador")) return "criador";
  if (n.includes("motion")) return "motion_designer";
  return "auditor";
}

const NIVEL: Record<Posta["cargo"], "alto" | "medio" | "baixo"> = {
  diretor_geral: "alto",
  gerente_setor: "alto",
  criador: "baixo",
  motion_designer: "medio",
  auditor: "medio",
};

export default function Relay({ postas }: { postas: Posta[] }) {
  const { d } = useIdioma();
  if (postas.length === 0) return null;

  const concluidas = postas.filter((p) => p.estado === "feito").length;
  const ativa = postas.findIndex((p) => p.estado === "ativo");
  // A linha acesa vai até o meio da posta em trabalho, não até o fim dela.
  const alcance = ativa >= 0 ? ativa + 0.5 : concluidas;
  const percurso = postas.length > 1 ? Math.min(alcance / (postas.length - 1), 1) : 0;
  // A linha da rede só é reservada quando alguma posta tem rede. Reservá-la
  // sempre abria um vão morto na configuração, que é justamente o estado em que
  // a trilha passa mais tempo na tela.
  const temRede = postas.some((p) => p.rede);

  return (
    <div
      className="relay"
      data-rede={temRede}
      style={
        {
          "--postas": postas.length,
          "--percurso": percurso,
        } as React.CSSProperties
      }
    >
      {postas.map((posta, i) => (
        <div className="posta" key={`${posta.passo}-${i}`} data-estado={posta.estado}>
          <span className="posta__marca" />
          <span className="posta__cargo">{d.roles[posta.cargo]}</span>
          {/* Quando alguma posta tem rede, a linha existe em todas: é o que
              mantém a fileira alinhada com uma só delas nomeada. */}
          {temRede && <span className="posta__rede">{posta.rede ?? "\u00a0"}</span>}
          <span className={posta.modelo ? "posta__modelo" : "posta__nivel"}>
            {posta.modelo ??
              posta.detalhe ??
              (NIVEL[posta.cargo] === "alto"
                ? d.models.tierHigh
                : NIVEL[posta.cargo] === "medio"
                  ? d.models.tierMid
                  : d.models.tierLow)}
          </span>
        </div>
      ))}
    </div>
  );
}
