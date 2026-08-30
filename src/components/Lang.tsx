import { useIdioma } from "../i18n";

/** Troca de idioma. Duas opções, sem menu: o custo de errar é um clique. */
export default function Lang() {
  const { idioma, trocar } = useIdioma();
  return (
    <div className="lang" role="group" aria-label="Idioma / Language">
      {(["pt", "en"] as const).map((op) => (
        <button
          key={op}
          onClick={() => trocar(op)}
          aria-pressed={idioma === op}
          aria-label={op === "pt" ? "Português" : "English"}
        >
          {op.toUpperCase()}
        </button>
      ))}
    </div>
  );
}
