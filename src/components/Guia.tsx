import { useIdioma } from "../i18n";
import { IconCheck } from "./Icons";

export interface Passo {
  id: string;
  rotulo: string;
  nota?: string;
  feito: boolean;
  /** Para onde levar quando a pessoa clica no passo que falta. */
  ir?: () => void;
}

/** Guia de primeiros passos, no pe do trilho.
 *
 *  Duas decisoes que vem do estudo de ativacao e valem a pena registrar:
 *
 *  O primeiro passo ja nasce cumprido. Ler a maquina e trabalho que o app fez
 *  sozinho antes de pedir qualquer coisa, e mostrar isso marcado coloca a
 *  pessoa dentro da jornada em vez de na linha de partida (endowed progress:
 *  cartao com dois selos ja carimbados fecha quase o dobro de um cartao vazio,
 *  com o mesmo esforco restante).
 *
 *  E a contagem diz quantos faltam, nao a porcentagem. "Falta 1" e uma
 *  distancia que se cobre; "75%" e um numero sobre o qual nao se age.
 *
 *  Some por inteiro quando termina: um guia que fica na tela depois de cumprido
 *  vira mobilia. */
export default function Guia({ passos }: { passos: Passo[] }) {
  const { d, f } = useIdioma();
  const faltam = passos.filter((p) => !p.feito);
  if (faltam.length === 0) return null;

  const proximo = faltam[0];

  return (
    <div className="guia">
      <div className="guia__topo">
        <span className="guia__titulo">{d.guide.title}</span>
        <span className="guia__conta">
          {faltam.length === 1
            ? f(d.guide.left, { n: faltam.length })
            : f(d.guide.leftMany, { n: faltam.length })}
        </span>
      </div>

      <ul className="guia__lista">
        {passos.map((p) => {
          const alvo = p.id === proximo.id;
          const conteudo = (
            <>
              <span className="guia__marca" aria-hidden>
                {p.feito && <IconCheck size={11} />}
              </span>
              <span className="guia__rotulo">{p.rotulo}</span>
            </>
          );
          return (
            <li key={p.id} className="guia__item" data-feito={p.feito} data-alvo={alvo}>
              {!p.feito && p.ir ? (
                <button type="button" onClick={p.ir} title={p.nota}>
                  {conteudo}
                </button>
              ) : (
                <span title={p.nota}>{conteudo}</span>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
