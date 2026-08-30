import React from "react";
import { AbsoluteFill, Audio, Sequence, staticFile } from "remotion";
import { loadFont } from "@remotion/fonts";
import { C, s } from "./tokens";
import {
  Abertura,
  Arte,
  Claude,
  Fecho,
  Inversao,
  Privacidade,
  Problema,
  Revezamento,
  Telas,
} from "./cenas";

// As mesmas fontes do aplicativo e do site. Carregadas do `public/` e não de
// um CDN: a renderização precisa ser determinística, e uma fonte que chega
// atrasada da rede troca a tipografia no meio do vídeo.
void loadFont({ family: "Geist", url: staticFile("geist.woff2"), weight: "100 900" });
void loadFont({ family: "GeistMono", url: staticFile("geist-mono.woff2"), weight: "100 900" });

/** As cenas, na ordem, com a duração de cada uma e a fala que a acompanha.
 *
 *  A duração NÃO é escolhida: é a da narração mais um respiro. Uma cena mais
 *  curta que a fala corta a locução no meio; mais longa deixa a tela parada
 *  esperando. O respiro extra vai onde a animação precisa de tempo próprio —
 *  a trilha de revezamento acendendo, as barras do comparativo crescendo.
 *
 *  Ficam numa lista e não em `<Sequence>` escritos à mão porque o `from` de
 *  cada cena é a soma das anteriores: calcular isso a mão é onde nasce o
 *  buraco de meio segundo entre duas cenas. */
const CENAS: { componente: React.FC; fala: string; segundos: number }[] = [
  { componente: Abertura,    fala: "01", segundos: 4.5 },
  { componente: Problema,    fala: "02", segundos: 5.9 },
  { componente: Revezamento, fala: "03", segundos: 6.0 },
  { componente: Inversao,    fala: "04", segundos: 6.0 },
  { componente: Telas,       fala: "05", segundos: 7.0 },
  { componente: Claude,      fala: "06", segundos: 4.4 },
  { componente: Arte,        fala: "07", segundos: 4.8 },
  { componente: Privacidade, fala: "08", segundos: 5.0 },
  { componente: Fecho,       fala: "09", segundos: 4.6 },
];

export const DURACAO_TOTAL = CENAS.reduce((t, c) => t + s(c.segundos), 0);

export const Apresentacao: React.FC = () => {
  let inicio = 0;
  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo }}>
      {/* A trilha corre por baixo de tudo, num volume que deixa a voz passar:
          medida na faixa de 1 a 4 kHz ela já fica 9 dB abaixo da narração, e
          a 18% ela vira fundo em vez de concorrente. O fade final evita o
          corte seco no último quadro. */}
      <Audio
        src={staticFile("audio/trilha.mp3")}
        volume={(q) =>
          q > DURACAO_TOTAL - s(1.4)
            ? Math.max(0, 0.18 * (1 - (q - (DURACAO_TOTAL - s(1.4))) / s(1.4)))
            : 0.18
        }
      />

      {CENAS.map(({ componente: Cena, fala, segundos }, i) => {
        const from = inicio;
        inicio += s(segundos);
        return (
          <Sequence key={i} from={from} durationInFrames={s(segundos)}>
            <Cena />
            {/* A fala entra com um quadro de atraso sobre o corte: começar
                exatamente no corte faz a primeira sílaba soar como parte da
                cena anterior. */}
            <Sequence from={s(0.25)}>
              <Audio src={staticFile(`audio/narracao/${fala}.mp3`)} />
            </Sequence>
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};
