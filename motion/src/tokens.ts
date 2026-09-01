/** Os tokens do aplicativo, não uma reinterpretação.
 *
 *  Cópia do `video/src/tokens.ts` de propósito, e não um import: os dois
 *  projetos têm `package.json` próprio e um import atravessando a fronteira
 *  faria o bundle de um depender da árvore do outro. Três superfícies com a
 *  mesma marca já é o padrão do projeto — o que não pode é uma delas divergir
 *  em silêncio, e por isso o valor está escrito e não interpretado. */
export const C = {
  fundo: "#15181C",
  cartao: "#1F2329",
  cartao2: "#2A2F36",
  afundado: "#111418",
  linha: "#2F3540",

  tinta: "#F7F8F9",
  tinta2: "#C3C9D1",
  tinta3: "#8D949E",

  acao: "#C9F227",
  acaoTinta: "#1B2306",
  acaoLavado: "#2E3A11",
} as const;

export const FONTE = "Geist, system-ui, -apple-system, Segoe UI, sans-serif";

/** 30 quadros por segundo. O roteiro fala em segundos; a conversão mora aqui. */
export const FPS = 30;

/** Segundos → quadros, com piso de 1.
 *
 *  O piso importa: uma cena de duração muito curta arredondaria para zero
 *  quadro, e o Remotion recusa uma sequência de duração zero — o render
 *  morreria no fim, depois de a pessoa já ter esperado. O `spec.rs` já barra
 *  isso antes, e este é o cinto além do suspensório. */
export const quadros = (segundos: number) => Math.max(1, Math.round(segundos * FPS));

/** A única curva de saída do projeto.
 *
 *  Curvas diferentes na mesma peça é o que faz um vídeo parecer montado por
 *  várias mãos. A apresentação do produto usa esta mesma. */
export const CURVA = [0.16, 1, 0.3, 1] as const;
