/** Marca da familia do modelo.
 *
 *  Glifos geometricos proprios, nao reproducoes das marcas registradas de cada
 *  laboratorio: replicar logo de terceiro num projeto que vai para o mundo e
 *  problema de licenca, e uma lista com nove logotipos coloridos vira ruido.
 *  A forma distingue a familia; a cor e a da propria marca, em dose pequena,
 *  para a lista poder ser varrida com o olho.
 */

const CORES: Record<string, string> = {
  Anthropic: "#C96442",
  Qwen: "#6E56CF",
  OpenAI: "#0F9D77",
  Meta: "#0064E0",
  Google: "#1A73E8",
  Mistral: "#EC650E",
  DeepSeek: "#4D6BFE",
  Microsoft: "#0097DC",
  IBM: "#0F62FE",
  Moonshot: "#3D3D57",
};

function Glifo({ familia }: { familia: string }) {
  switch (familia) {
    // Anthropic: duas hastes inclinadas e a trave, o "A" reduzido ao que
    // sobrevive a 15px. Formas cheias, como o resto do conjunto: contorno de
    // 1px some no primeiro tamanho pequeno.
    case "Anthropic":
      return (
        <g>
          <path d="M2.2 13.4 L6.6 2.6 h1.9 L4.1 13.4 z" />
          <path d="M11.9 13.4 L7.5 2.6 h1.9 L13.8 13.4 z" />
          <path d="M4.9 9.4 h6.2 l0.75 1.9 h-7.7 z" />
        </g>
      );
    case "Qwen":
      // Losango com corte: a forma angular fechada da familia.
      return <path d="M8 1.5 14.5 8 8 14.5 1.5 8z M8 5.2 5.2 8 8 10.8 10.8 8z" fillRule="evenodd" />;
    case "OpenAI":
      // Anel hexagonal aberto.
      return (
        <path
          d="M8 1.6l5.5 3.2v6.4L8 14.4 2.5 11.2V4.8z"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.7"
          strokeLinejoin="round"
        />
      );
    case "Meta":
      // Duas voltas encadeadas.
      return (
        <path
          d="M2.4 10.2c0-3 1.6-5.2 3.3-5.2 2.2 0 3.1 3.4 4.6 5.2 1 1.2 3.3 1 3.3-1.6"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      );
    case "Google":
      // Circulo aberto, como um arco incompleto.
      return (
        <path
          d="M13.2 5.4A6 6 0 1 0 14 8h-6"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      );
    case "Mistral":
      // Faixas horizontais, densidade decrescente.
      return (
        <g>
          <rect x="2" y="3" width="12" height="2.6" rx="0.6" />
          <rect x="2" y="6.7" width="8.5" height="2.6" rx="0.6" opacity="0.7" />
          <rect x="2" y="10.4" width="5" height="2.6" rx="0.6" opacity="0.45" />
        </g>
      );
    case "DeepSeek":
      // Nucleo com orbita.
      return (
        <g>
          <circle cx="8" cy="8" r="2.6" />
          <ellipse cx="8" cy="8" rx="6.2" ry="3" fill="none" stroke="currentColor" strokeWidth="1.4" transform="rotate(-28 8 8)" />
        </g>
      );
    case "Microsoft":
      // Quatro modulos.
      return (
        <g>
          <rect x="2.2" y="2.2" width="5.4" height="5.4" rx="0.7" />
          <rect x="8.4" y="2.2" width="5.4" height="5.4" rx="0.7" opacity="0.72" />
          <rect x="2.2" y="8.4" width="5.4" height="5.4" rx="0.7" opacity="0.72" />
          <rect x="8.4" y="8.4" width="5.4" height="5.4" rx="0.7" opacity="0.45" />
        </g>
      );
    case "IBM":
      // Listras, a assinatura visual da casa.
      return (
        <g>
          <rect x="1.8" y="3.4" width="12.4" height="1.7" rx="0.5" />
          <rect x="1.8" y="7.15" width="12.4" height="1.7" rx="0.5" />
          <rect x="1.8" y="10.9" width="12.4" height="1.7" rx="0.5" />
        </g>
      );
    case "Moonshot":
      // Crescente.
      return <path d="M10.6 2.4a6 6 0 1 0 2.6 8.9A6.6 6.6 0 0 1 10.6 2.4z" />;
    default:
      return <circle cx="8" cy="8" r="3.4" />;
  }
}

export default function MarcaModelo({ familia, size = 16 }: { familia: string; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill={CORES[familia] ?? "currentColor"}
      color={CORES[familia] ?? "currentColor"}
      role="img"
      aria-label={familia}
      style={{ flex: "none" }}
    >
      <title>{familia}</title>
      <Glifo familia={familia} />
    </svg>
  );
}
