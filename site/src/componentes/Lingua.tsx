import { useIdioma, type Idioma } from "../i18n";

/** PT · EN, na mesma forma do seletor que existe dentro do aplicativo.
 *
 *  Aparece mesmo quando o idioma detectado já é o certo: quem chega numa
 *  página no idioma errado precisa ver a saída, e quem chega na certa não é
 *  incomodado por dois botões pequenos no canto. */
export default function Lingua() {
  const { idioma, trocar, d } = useIdioma();

  return (
    <div className="lingua" role="group" aria-label={d.nav.idioma}>
      {(["pt", "en"] as Idioma[]).map((i) => (
        <button
          key={i}
          className="lingua__op"
          aria-pressed={idioma === i}
          onClick={() => trocar(i)}
        >
          {i.toUpperCase()}
        </button>
      ))}
    </div>
  );
}
