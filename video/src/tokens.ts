/** Os tokens do aplicativo, não uma reinterpretação.
 *
 *  O vídeo vai no site e o site tem a cara do app: três superfícies
 *  diferentes para a mesma marca seria confusão, não riqueza. */
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

export const FONTE = "Geist";
export const MONO = "GeistMono";

/** 30 quadros por segundo: `s(2)` lê melhor que `60` no meio de uma cena. */
export const FPS = 30;
export const s = (segundos: number) => Math.round(segundos * FPS);
