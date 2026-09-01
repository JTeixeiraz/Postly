/** A câmera: como a direção de uma cena vira `transform` num quadro.
 *
 *  Fora do `cenas.tsx` porque é o cálculo, e o `cenas.tsx` é a composição. E
 *  porque é a parte que decide se o vídeo parece dirigido ou parece template —
 *  vale ler sozinha. */

import { interpolate } from "remotion";
import type { Direcao, Foco, Look, Movimento } from "./tipos";

/** Quanto a câmera anda, do mínimo ao máximo de energia.
 *
 *  O piso não é zero: uma câmera parada num vídeo feito de fotos é uma
 *  apresentação de slides, e a energia baixa deve significar "calmo", não
 *  "morto". O teto é 0,26 porque acima disso a imagem perde resolução visível
 *  no zoom — medido no primeiro render, onde 1,4× já mostrava o grão. */
const AMPLITUDE_MIN = 0.06;
const AMPLITUDE_MAX = 0.26;

/** O ponto da imagem que fica fixo enquanto a câmera se move.
 *
 *  Em porcentagem de `transform-origin`. Aproximar do topo é enquadramento
 *  diferente de aproximar do centro na mesma foto — é por isso que `foco` é um
 *  campo separado de `movimento`. */
function origem(foco: Foco): string {
  switch (foco) {
    case "topo":
      return "50% 12%";
    case "base":
      return "50% 88%";
    case "esquerda":
      return "12% 50%";
    case "direita":
      return "88% 50%";
    default:
      return "50% 50%";
  }
}

/** A escala inicial de um movimento de varredura.
 *
 *  Varrer exige folga: sem escala maior que 1 a imagem sairia da moldura e
 *  deixaria barra preta na borda de onde ela veio. */
function escalaBase(mov: Movimento, amp: number): number {
  switch (mov) {
    case "aproximar":
      return 1;
    case "afastar":
      return 1 + amp;
    case "nenhum":
      return 1;
    // Varredura e deslocamento vertical: a folga é o próprio deslocamento.
    default:
      return 1 + amp;
  }
}

/** O `transform` desta cena neste quadro. */
export function camera(
  direcao: Direcao,
  look: Look,
  frame: number,
  total: number
): { transform: string; transformOrigin: string } {
  const amp = interpolate(look.energia, [0, 1], [AMPLITUDE_MIN, AMPLITUDE_MAX], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const t = total <= 1 ? 0 : frame / (total - 1);

  const base = escalaBase(direcao.movimento, amp);
  let escala = base;
  let x = 0;
  let y = 0;

  switch (direcao.movimento) {
    case "aproximar":
      escala = 1 + amp * t;
      break;
    case "afastar":
      escala = 1 + amp * (1 - t);
      break;
    // O deslocamento é em % da própria caixa, e vai a metade da amplitude: a
    // outra metade é a folga que a escala criou. Passar disso mostraria a borda.
    case "varrer_esquerda":
      x = -amp * 50 * t;
      break;
    case "varrer_direita":
      x = amp * 50 * t;
      break;
    case "subir":
      y = -amp * 50 * t;
      break;
    case "descer":
      y = amp * 50 * t;
      break;
    case "nenhum":
      break;
  }

  return {
    transform: `scale(${escala}) translate(${x}%, ${y}%)`,
    transformOrigin: origem(direcao.foco),
  };
}

/** A entrada da cena, como opacidade e deslocamento.
 *
 *  `corte` devolve opacidade 1 do primeiro quadro: um corte seco entre duas
 *  cenas é uma escolha de ritmo, não a ausência de uma transição. Sem esta
 *  opção todo vídeo teria o mesmo pulso de fade e nada acentuaria nada. */
export function entrada(
  direcao: Direcao,
  look: Look,
  frame: number,
  total: number,
  curva: (x: number) => number
): { opacity: number; transform?: string; clipPath?: string } {
  // Energia alta encurta a transição: 8 a 16 quadros. Uma transição de duração
  // fixa faria a mesma abertura num vídeo calmo e num agressivo.
  const passo = Math.round(interpolate(look.energia, [0, 1], [16, 8]));
  const t = Math.min(passo, Math.floor(total / 3));
  if (t <= 0 || direcao.entrada === "corte") {
    // Ainda apaga no fim, senão a última cena termina em corte para preto.
    return { opacity: frame >= total - 2 ? 0 : 1 };
  }

  const p = curva(
    interpolate(frame, [0, t], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
    })
  );
  const saida = interpolate(frame, [total - t, total], [1, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });
  const opacity = Math.min(p, saida);

  switch (direcao.entrada) {
    case "subir":
      return { opacity, transform: `translateY(${(1 - p) * 4}%)` };
    case "escala":
      return { opacity, transform: `scale(${0.94 + p * 0.06})` };
    case "cortina":
      // A cortina não mexe na opacidade da imagem: ela revela. Somar os dois
      // daria um fade com uma máscara por cima, que lê como defeito.
      return { opacity: saida, clipPath: `inset(0 ${(1 - p) * 100}% 0 0)` };
    default:
      return { opacity };
  }
}
