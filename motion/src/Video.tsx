import { loadFont } from "@remotion/fonts";
import { AbsoluteFill, Audio, continueRender, delayRender, Sequence, staticFile } from "remotion";
import { C, quadros } from "./tokens";
// A fonte entra pelo BUNDLE, e não por `staticFile` como no vídeo de
// apresentação. A razão é que aqui o `publicDir` do bundle é a pasta do
// projeto de quem usa — `staticFile("geist.woff2")` procuraria a fonte no meio
// das fotos dela, e não acharia. O import faz o bundler emitir o arquivo e
// devolver a URL dele.
import geist from "./geist.woff2";

// O RENDER ESPERA A FONTE, e é para isso que `delayRender` existe: sem ele o
// Remotion começa a capturar quadros enquanto a promessa do `loadFont` ainda
// está pendente, e os primeiros sairiam no fallback.
//
// Que a Geist chega de fato FOI MEDIDO, e por controle: com `fontFamily`
// apontando para uma família inexistente o mesmo quadro sai em serif; com a
// Geist sai na grotesca, e o navegador do render responde
// `document.fonts.check("96px Geist") === true`. A corrida em si não foi
// observada nesta máquina — o `delayRender` está aqui porque é o contrato
// documentado para trabalho assíncrono antes do primeiro quadro, não porque um
// render tenha falhado.
//
// Fora do componente de propósito: dentro dele, a fonte seria carregada uma vez
// por quadro — 180 vezes num vídeo de 6 segundos.
const esperandoAFonte = delayRender("carregando a Geist");
loadFont({ family: "Geist", url: geist, weight: "100 900" })
  .then(() => continueRender(esperandoAFonte))
  // Sem tipografia da marca o vídeo ainda é um vídeo; travado no
  // `delayRender` ele não é nada. A falha libera o render e segue no fallback.
  // Sem a tipografia da marca o vídeo ainda é um vídeo; travado no
  // `delayRender` ele não é nada. A falha libera o render e segue no fallback.
  .catch(() => continueRender(esperandoAFonte));
import { Palco } from "./cenas";
import type { Props } from "./tipos";

/** A montagem: cenas em sequência, áudio por cima.
 *
 *  A duração total é a soma das cenas, calculada aqui e também no Rust
 *  (`Roteiro::duracao_s`). Os dois precisam concordar, e concordam por
 *  construção: os dois somam `dur_s`. O que NÃO pode acontecer é um deles
 *  arredondar antes de somar — daí a soma em segundos e a conversão para
 *  quadros só no fim. */
export function duracaoEmQuadros(props: Props) {
  return Math.max(
    1,
    props.roteiro.cenas.reduce((total, c) => total + quadros(c.dur_s), 0)
  );
}

export const Video: React.FC<Props> = ({ roteiro, assets, narracao }) => {
  let inicio = 0;

  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo }}>
      {roteiro.cenas.map((cena, i) => {
        const dur = quadros(cena.dur_s);
        const de = inicio;
        inicio += dur;
        return (
          <Sequence key={i} from={de} durationInFrames={dur}>
            <Palco cena={cena} assets={assets} look={roteiro.look} />
          </Sequence>
        );
      })}

      {/* A narração entra em sequência, na ordem dos arquivos. Não há mixagem
          por cena: o roteiro já distribuiu o TEXTO da voz pelas cenas, e o
          áudio gravado é contínuo. Cortá-lo por cena produziria emenda audível
          em cada transição. */}
      {narracao.map((src, i) => (
        <Audio key={`vo-${i}`} src={staticFile(src)} />
      ))}

      {/* A trilha a 18% — o mesmo valor medido para o vídeo de apresentação do
          Postly, onde a voz saiu 20 a 25 dB à frente. Sem narração ela pode
          subir, porque não há nada para mascarar. */}
      {roteiro.trilha && assets[roteiro.trilha] && (
        <Audio
          src={staticFile(assets[roteiro.trilha])}
          volume={narracao.length ? 0.18 : 0.45}
        />
      )}
    </AbsoluteFill>
  );
};
