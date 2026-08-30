import React from "react";
import { AbsoluteFill, Sequence } from "remotion";
import { loadFont } from "@remotion/fonts";
import { staticFile } from "remotion";
import { C, s } from "./tokens";
import {
  Abertura,
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

/** As cenas, na ordem, com a duração de cada uma.
 *
 *  Ficam numa lista e não em `<Sequence>` escritos à mão porque o `from` de
 *  cada cena é a soma das anteriores: calcular isso a mão é onde nasce o
 *  buraco de meio segundo entre duas cenas. */
const CENAS: { componente: React.FC; segundos: number }[] = [
  { componente: Abertura, segundos: 4.2 },
  { componente: Problema, segundos: 4.6 },
  { componente: Revezamento, segundos: 7.0 },
  { componente: Inversao, segundos: 6.4 },
  { componente: Telas, segundos: 6.6 },
  { componente: Privacidade, segundos: 6.2 },
  { componente: Fecho, segundos: 5.0 },
];

export const DURACAO_TOTAL = CENAS.reduce((t, c) => t + s(c.segundos), 0);

export const Apresentacao: React.FC = () => {
  let inicio = 0;
  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo }}>
      {CENAS.map(({ componente: Cena, segundos }, i) => {
        const from = inicio;
        inicio += s(segundos);
        return (
          <Sequence key={i} from={from} durationInFrames={s(segundos)}>
            <Cena />
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};
