/** O roteiro, do jeito que o Rust o serializa.
 *
 *  Espelho tipado do `src-tauri/src/video/spec.rs`, mantido à mão pela mesma
 *  razão que `src/types.ts` é: o contrato é pequeno demais para valer um
 *  gerador, e um gerador seria uma dependência a mais no build do render. */

export type TipoCena =
  | "titulo"
  | "ken_burns"
  | "placa"
  | "comparacao"
  | "declaracao"
  | "fecho"
  | "clipe";

/** O trecho de um vídeo que a pessoa gravou. */
export interface Corte {
  /** Nome do arquivo em `clipes/`, não caminho. */
  arquivo: string;
  de_s: number;
  ate_s: number;
}

export type Movimento =
  | "aproximar"
  | "afastar"
  | "varrer_esquerda"
  | "varrer_direita"
  | "subir"
  | "descer"
  | "nenhum";

export type Foco = "centro" | "topo" | "base" | "esquerda" | "direita";

export type Pouso =
  | "inferior_esquerda"
  | "inferior_direita"
  | "superior_esquerda"
  | "centro"
  | "coluna_esquerda";

export type Entrada = "fade" | "subir" | "escala" | "cortina" | "corte";

/** COMO a cena se parece. É o que separa este sistema de um template: sem ela,
 *  duas cenas do mesmo tipo sairiam idênticas no olhar por mais que a montagem
 *  mudasse. O Rust normaliza e apara antes de chegar aqui, então todo campo
 *  está preenchido e dentro da faixa. */
export interface Direcao {
  movimento: Movimento;
  foco: Foco;
  pouso: Pouso;
  entrada: Entrada;
  escala_texto: number;
}

/** A direção do vídeo inteiro, que cascateia por cima das cenas. */
export interface Look {
  /** 0 = quase parado, 1 = agressivo. Multiplica o deslocamento da câmera e
   *  encurta as transições. */
  energia: number;
  vinheta: boolean;
  filete: boolean;
}

export interface Cena {
  tipo: TipoCena;
  /** Segundos. A conversão para quadros é do render, não do modelo. */
  dur_s: number;
  titulo: string;
  subtitulo: string;
  /** Nomes de arquivo, não caminhos. O `staticFile` resolve. */
  imagens: string[];
  narracao: string;
  /** Preenchido quando `tipo` é `clipe`. */
  corte: Corte | null;
  direcao: Direcao;
}

export interface Roteiro {
  cenas: Cena[];
  trilha: string;
  proporcao: string;
  racional: string;
  look: Look;
}

/** O que o render passa para a composição.
 *
 *  As URLs dos assets vêm resolvidas de fora porque quem sabe onde a pasta do
 *  projeto está é o sidecar, não a composição. Uma composição que montasse
 *  caminho sozinha só funcionaria com uma convenção de pastas fixa. */
export interface Props {
  roteiro: Roteiro;
  /** nome do arquivo → URL que o Remotion consegue carregar. */
  assets: Record<string, string>;
  /** Os arquivos de narração, na ordem em que devem tocar. */
  narracao: string[];
}
