import React from "react";
import { Easing, interpolate, useCurrentFrame } from "remotion";
import { C, FONTE } from "./tokens";

/** Saída exponencial, a mesma curva do app e do site.
 *
 *  Toda animação daqui passa por esta função em vez de escolher a própria
 *  curva: curvas diferentes na mesma peça é o que faz um vídeo parecer montado
 *  por várias mãos. */
export const saida = Easing.bezier(0.16, 1, 0.3, 1);

/** Entrada padrão: sobe alguns pixels e aparece.
 *
 *  `atraso` é em quadros e conta a partir do início da Sequence em que a peça
 *  vive, não do vídeo inteiro. */
export function useEntrada(atraso = 0, distancia = 22, duracao = 20) {
  const quadro = useCurrentFrame();
  const t = interpolate(quadro, [atraso, atraso + duracao], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: saida,
  });
  return { opacity: t, transform: `translateY(${(1 - t) * distancia}px)` };
}

/** A marca: a haste do P é a rota, o ponto é a posta onde o despacho está. */
export const Marca: React.FC<{ tamanho?: number; cor?: string }> = ({
  tamanho = 64,
  cor = C.acao,
}) => (
  <svg
    width={Math.round(tamanho * (14 / 21.4))}
    height={tamanho}
    viewBox="9 5.6 14 21.4"
    style={{ overflow: "visible" }}
  >
    <path
      d="M11.6 24.5V9.4h5.1a3.9 3.9 0 0 1 0 7.8h-5.1"
      stroke={cor}
      strokeWidth={2.6}
      fill="none"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
    <circle cx="11.6" cy="17.2" r="2.5" fill={cor} />
  </svg>
);

/** Título de cena. Uma ideia dominante por cena, e ela é este texto. */
export const Titulo: React.FC<{
  children: React.ReactNode;
  atraso?: number;
  tamanho?: number;
  largura?: number;
}> = ({ children, atraso = 0, tamanho = 86, largura = 1300 }) => {
  const e = useEntrada(atraso);
  return (
    <h1
      style={{
        ...e,
        margin: 0,
        fontFamily: FONTE,
        fontWeight: 680,
        fontSize: tamanho,
        lineHeight: 1.04,
        letterSpacing: "-0.035em",
        color: C.tinta,
        maxWidth: largura,
      }}
    >
      {children}
    </h1>
  );
};

export const Linha: React.FC<{
  children: React.ReactNode;
  atraso?: number;
  tamanho?: number;
  cor?: string;
  largura?: number;
}> = ({ children, atraso = 0, tamanho = 34, cor = C.tinta2, largura = 1040 }) => {
  const e = useEntrada(atraso);
  return (
    <p
      style={{
        ...e,
        margin: 0,
        fontFamily: FONTE,
        fontWeight: 400,
        fontSize: tamanho,
        lineHeight: 1.45,
        color: cor,
        maxWidth: largura,
      }}
    >
      {children}
    </p>
  );
};

/** O brilho de canto que o site também usa. Assina a superfície sem
 *  competir com o conteúdo. */
export const Brilho: React.FC<{ x?: string; y?: string; raio?: number }> = ({
  x = "82%",
  y = "12%",
  raio = 900,
}) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      width: raio,
      height: raio,
      transform: "translate(-50%, -50%)",
      background: `radial-gradient(circle, rgba(201,242,39,0.10) 0%, rgba(201,242,39,0) 62%)`,
      pointerEvents: "none",
    }}
  />
);
