import { Composition } from "remotion";
import { Video, duracaoEmQuadros } from "./Video";
import { FPS } from "./tokens";
import type { Direcao, Props } from "./tipos";

const DIR: Direcao = {
  movimento: "aproximar",
  foco: "centro",
  pouso: "centro",
  entrada: "fade",
  escala_texto: 1,
};

/** O roteiro de exemplo, para o Remotion Studio abrir sem o app.
 *
 *  Não é o padrão de produção: o sidecar SEMPRE passa `inputProps` com o
 *  roteiro de verdade. Isto existe para conferir uma cena à mão sem precisar
 *  rodar uma campanha inteira. */
const EXEMPLO: Props = {
  roteiro: {
    cenas: [
      { tipo: "titulo", dur_s: 2.5, titulo: "Um título", subtitulo: "e um subtítulo", imagens: [], narracao: "", corte: null, direcao: DIR },
      { tipo: "declaracao", dur_s: 3, titulo: "A frase que é o conteúdo.", subtitulo: "", imagens: [], narracao: "", corte: null, direcao: { ...DIR, pouso: "coluna_esquerda" } },
      { tipo: "fecho", dur_s: 2.5, titulo: "O fecho", subtitulo: "com chamada", imagens: [], narracao: "", corte: null, direcao: DIR },
    ],
    trilha: "",
    proporcao: "16:9",
    racional: "exemplo",
    look: { energia: 0.5, vinheta: false, filete: true },
  },
  assets: {},
  narracao: [],
};

export const RemotionRoot: React.FC = () => (
  <Composition
    id="VideoDoUsuario"
    component={Video}
    // Recalculada a cada roteiro: a duração é do conteúdo, não da composição.
    // Sem isto o Remotion cortaria todo vídeo no tamanho do exemplo.
    calculateMetadata={({ props }: { props: Props }) => ({
      durationInFrames: duracaoEmQuadros(props),
    })}
    defaultProps={EXEMPLO}
    durationInFrames={duracaoEmQuadros(EXEMPLO)}
    fps={FPS}
    width={1920}
    height={1080}
  />
);
