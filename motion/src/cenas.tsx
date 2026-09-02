import {
  AbsoluteFill,
  Img,
  OffthreadVideo,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { C, CURVA, FONTE } from "./tokens";
import { camera, entrada } from "./camera";
import type { Cena, Look, Pouso } from "./tipos";

/** As cenas do vocabulário fechado.
 *
 *  Fechado porque o Motion Designer escolhe entre elas, não inventa — as três
 *  razões estão no topo do `spec.rs`. O que ele NÃO tem preso é a aparência:
 *  cada cena recebe uma `direcao` (de onde a câmera parte, para onde vai, onde
 *  o texto pousa, como a cena entra) e um `look` do vídeo inteiro. É isso que
 *  separa duas `ken_burns` uma da outra. */

/** A curva de saída do projeto, avaliada.
 *
 *  Aproximação por Newton do x da cúbica de Bézier: `Easing.bezier` do Remotion
 *  faz o mesmo, mas importá-lo aqui traria a dependência de easing para dentro
 *  de um arquivo que só precisa de um número. */
function bezier(x: number) {
  const [x1, y1, x2, y2] = CURVA;
  let t = x;
  for (let i = 0; i < 5; i++) {
    const cx = 3 * (1 - t) * (1 - t) * t * x1 + 3 * (1 - t) * t * t * x2 + t * t * t;
    const d =
      3 * (1 - t) * (1 - t) * x1 + 6 * (1 - t) * t * (x2 - x1) + 3 * t * t * (1 - x2);
    if (Math.abs(d) < 1e-5) break;
    t -= (cx - x) / d;
  }
  return 3 * (1 - t) * (1 - t) * t * y1 + 3 * (1 - t) * t * t * y2 + t * t * t;
}

/** Nome de arquivo → URL que o render consegue carregar.
 *
 *  `staticFile` e não `file://`: medido, o Chromium do render recusa URLs de
 *  arquivo local com "Not allowed to load local resource", e a falha só aparece
 *  no meio do render. O sidecar aponta o `publicDir` do bundle para a pasta do
 *  projeto, e é isso que faz o caminho relativo resolver.
 *
 *  Devolve `undefined` para nome ausente em vez de estourar: o `spec.rs` já
 *  recusa roteiro que cite imagem inexistente, e um quadro sem foto é melhor
 *  que um render que morre. */
function arquivo(assets: Record<string, string>, nome: string | undefined) {
  const rel = nome ? assets[nome] : undefined;
  return rel ? staticFile(rel) : undefined;
}

/** Onde o bloco de texto pousa no quadro.
 *
 *  Devolve o alinhamento da caixa inteira, não só do texto: mover o texto para
 *  a direita sem mover a caixa deixaria a tarja do lado errado. */
function pouso(p: Pouso): React.CSSProperties {
  switch (p) {
    case "inferior_direita":
      return { justifyContent: "flex-end", alignItems: "flex-end", textAlign: "right" };
    case "superior_esquerda":
      return { justifyContent: "flex-start", alignItems: "flex-start" };
    case "centro":
      return { justifyContent: "center", alignItems: "center", textAlign: "center" };
    case "coluna_esquerda":
      // Coluna estreita colada na borda: o texto vira bloco vertical em vez de
      // faixa horizontal, e a leitura do quadro inteiro muda.
      return { justifyContent: "center", alignItems: "flex-start", maxWidth: "42%" };
    default:
      return { justifyContent: "flex-end", alignItems: "flex-start" };
  }
}

/** O véu por trás do texto, na direção em que ele pousou.
 *
 *  Gradiente e não cor chapada: chapado apaga a foto, e a foto é o conteúdo da
 *  cena. O gradiente escurece só onde o texto precisa vencer — a mesma correção
 *  que o pôster do vídeo de apresentação recebeu.
 *
 *  A direção do gradiente segue o pouso. Um véu que vem sempre de baixo com o
 *  texto no topo deixaria o texto sem contraste e a foto escura no lado errado. */
function veu(p: Pouso): string {
  const de = "rgba(17,20,24,0.92)";
  const nada = "rgba(17,20,24,0)";
  switch (p) {
    case "superior_esquerda":
      return `linear-gradient(to bottom, ${de} 0%, ${nada} 45%)`;
    case "coluna_esquerda":
      return `linear-gradient(to right, ${de} 0%, ${nada} 60%)`;
    case "centro":
      return `radial-gradient(ellipse at center, ${de} 0%, ${nada} 70%)`;
    default:
      return `linear-gradient(to top, ${de} 0%, ${nada} 45%)`;
  }
}

const base: React.CSSProperties = {
  backgroundColor: C.fundo,
  color: C.tinta,
  fontFamily: FONTE,
  justifyContent: "center",
  alignItems: "center",
  padding: "0 8%",
  textAlign: "center",
};

export function Palco({
  cena,
  assets,
  look,
}: {
  cena: Cena;
  assets: Record<string, string>;
  look: Look;
}) {
  const frame = useCurrentFrame();
  const { durationInFrames: total } = useVideoConfig();
  const ent = entrada(cena.direcao, look, frame, total, bezier);

  const envelope: React.CSSProperties = {
    opacity: ent.opacity,
    transform: ent.transform,
    clipPath: ent.clipPath,
  };

  switch (cena.tipo) {
    case "ken_burns":
    case "placa":
      return (
        <ComImagem
          cena={cena}
          assets={assets}
          look={look}
          envelope={envelope}
          frame={frame}
          total={total}
          conter={cena.tipo === "placa"}
        />
      );
    case "clipe":
      return <Clipe cena={cena} envelope={envelope} look={look} />;
    case "comparacao":
      return (
        <Comparacao
          cena={cena}
          assets={assets}
          look={look}
          envelope={envelope}
          frame={frame}
          total={total}
        />
      );
    case "fecho":
      return <Fecho cena={cena} look={look} envelope={envelope} />;
    case "declaracao":
      return <SoTexto cena={cena} look={look} envelope={envelope} corpo={72} />;
    // `titulo` é o padrão e não um `case` próprio: se um roteiro chegasse com
    // um tipo que o TypeScript não conhece (versão do Rust à frente da do
    // bundle), um quadro preto seria pior que um título legível.
    default:
      return <SoTexto cena={cena} look={look} envelope={envelope} corpo={96} />;
  }
}

/** Ken Burns e Placa são a mesma cena com `objectFit` diferente.
 *
 *  Separá-las em dois componentes duplicaria a câmera, a tarja e o véu — três
 *  coisas que precisam mudar juntas. O que de fato difere é se a foto preenche
 *  ou cabe. */
function ComImagem({
  cena,
  assets,
  look,
  envelope,
  frame,
  total,
  conter,
}: {
  cena: Cena;
  assets: Record<string, string>;
  look: Look;
  envelope: React.CSSProperties;
  frame: number;
  total: number;
  conter: boolean;
}) {
  const src = arquivo(assets, cena.imagens[0]);
  // Uma placa é para ver a foto inteira: mover a câmera nela cortaria
  // justamente o que ela existe para mostrar.
  const cam = conter
    ? { transform: undefined, transformOrigin: undefined }
    : camera(cena.direcao, look, frame, total);

  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo, ...envelope }}>
      {src && (
        <Img
          src={src}
          style={{
            width: "100%",
            height: "100%",
            objectFit: conter ? "contain" : "cover",
            transform: cam.transform,
            transformOrigin: cam.transformOrigin,
          }}
        />
      )}
      {look.vinheta && <Vinheta />}
      <Tarja cena={cena} look={look} />
    </AbsoluteFill>
  );
}

/** Um trecho do vídeo que a pessoa gravou.
 *
 *  `OffthreadVideo` e não `Video`: o render acontece fora do navegador, e o
 *  `Video` depende do relógio da tag `<video>`, que num render determinístico
 *  não avança sozinho. O Offthread extrai o quadro exato pelo compositor.
 *
 *  `trimBefore`/`trimAfter` em QUADROS, e o roteiro fala em segundos — a
 *  conversão mora aqui, que é onde o fps é conhecido. Deixar o modelo declarar
 *  quadro seria pedir uma multiplicação que ele erra. */
function Clipe({
  cena,
  envelope,
  look,
}: {
  cena: Cena;
  envelope: React.CSSProperties;
  look: Look;
}) {
  const { fps } = useVideoConfig();
  if (!cena.corte) return <AbsoluteFill style={{ backgroundColor: C.fundo }} />;

  const de = Math.max(0, Math.round(cena.corte.de_s * fps));
  const ate = Math.max(de + 1, Math.round(cena.corte.ate_s * fps));

  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo, ...envelope }}>
      <OffthreadVideo
        src={staticFile(`clipes/${cena.corte.arquivo}`)}
        trimBefore={de}
        trimAfter={ate}
        style={{ width: "100%", height: "100%", objectFit: "cover" }}
      />
      {look.vinheta && <Vinheta />}
      <Tarja cena={cena} look={look} />
    </AbsoluteFill>
  );
}

function Comparacao({
  cena,
  assets,
  look,
  envelope,
  frame,
  total,
}: {
  cena: Cena;
  assets: Record<string, string>;
  look: Look;
  envelope: React.CSSProperties;
  frame: number;
  total: number;
}) {
  const cam = camera(cena.direcao, look, frame, total);
  return (
    <AbsoluteFill style={{ backgroundColor: C.fundo, flexDirection: "row", ...envelope }}>
      {[0, 1].map((i) => {
        const src = arquivo(assets, cena.imagens[i]);
        return (
          <div key={i} style={{ flex: 1, overflow: "hidden", position: "relative" }}>
            {src && (
              <Img
                src={src}
                style={{
                  width: "100%",
                  height: "100%",
                  objectFit: "cover",
                  // Os dois lados andam em direções opostas: com a mesma câmera
                  // a comparação lê como uma foto só cortada ao meio.
                  transform: i === 0 ? cam.transform : espelhar(cam.transform),
                  transformOrigin: cam.transformOrigin,
                }}
              />
            )}
          </div>
        );
      })}
      {look.vinheta && <Vinheta />}
      <Tarja cena={cena} look={look} />
    </AbsoluteFill>
  );
}

/** Inverte o sinal do deslocamento horizontal do `transform`.
 *
 *  Só o `translate` X: espelhar a escala faria um lado crescer enquanto o outro
 *  encolhe, e a comparação viraria um efeito em vez de uma comparação. */
function espelhar(t: string | undefined) {
  return t?.replace(/translate\((-?[\d.]+)%/, (_, x) => `translate(${-Number(x)}%`);
}

function SoTexto({
  cena,
  look,
  envelope,
  corpo,
}: {
  cena: Cena;
  look: Look;
  envelope: React.CSSProperties;
  corpo: number;
}) {
  const e = cena.direcao.escala_texto;
  const alinhado = pouso(cena.direcao.pouso);
  return (
    <AbsoluteFill
      style={{
        ...base,
        ...alinhado,
        textAlign: alinhado.textAlign ?? "center",
        ...envelope,
      }}
    >
      {look.filete && <Filete />}
      <h1
        style={{
          fontSize: corpo * e,
          fontWeight: 600,
          margin: 0,
          lineHeight: 1.1,
          letterSpacing: "-0.03em",
        }}
      >
        {cena.titulo}
      </h1>
      {cena.subtitulo && (
        <p style={{ fontSize: 40 * e, color: C.tinta2, marginTop: 24 }}>{cena.subtitulo}</p>
      )}
      {look.vinheta && <Vinheta />}
    </AbsoluteFill>
  );
}

function Fecho({
  cena,
  look,
  envelope,
}: {
  cena: Cena;
  look: Look;
  envelope: React.CSSProperties;
}) {
  const e = cena.direcao.escala_texto;
  const alinhado = pouso(cena.direcao.pouso);
  return (
    <AbsoluteFill
      style={{ ...base, ...alinhado, textAlign: alinhado.textAlign ?? "center", ...envelope }}
    >
      <h2
        style={{ fontSize: 80 * e, fontWeight: 600, margin: 0, letterSpacing: "-0.03em" }}
      >
        {cena.titulo}
      </h2>
      {cena.subtitulo && (
        // O lime como FUNDO, com tinta escura por cima. Como texto ele cai para
        // 1,2:1 em papel — a mesma lição que o tema claro do app ensinou.
        <span
          style={{
            marginTop: 40,
            padding: `${18 * e}px ${40 * e}px`,
            borderRadius: 999,
            backgroundColor: C.acao,
            color: C.acaoTinta,
            fontSize: 36 * e,
            fontWeight: 600,
          }}
        >
          {cena.subtitulo}
        </span>
      )}
      {look.vinheta && <Vinheta />}
    </AbsoluteFill>
  );
}

/** A faixa de texto sobre imagem, na direção em que o texto pousou. */
function Tarja({ cena, look }: { cena: Cena; look: Look }) {
  if (!cena.titulo && !cena.subtitulo) return null;
  const e = cena.direcao.escala_texto;
  const alinhado = pouso(cena.direcao.pouso);
  return (
    <AbsoluteFill
      style={{
        display: "flex",
        flexDirection: "column",
        padding: "6%",
        background: veu(cena.direcao.pouso),
        ...alinhado,
      }}
    >
      {look.filete && <Filete />}
      {cena.titulo && (
        <h2
          style={{
            fontFamily: FONTE,
            color: C.tinta,
            fontSize: 56 * e,
            fontWeight: 600,
            margin: 0,
            lineHeight: 1.1,
            letterSpacing: "-0.02em",
          }}
        >
          {cena.titulo}
        </h2>
      )}
      {cena.subtitulo && (
        <p style={{ fontFamily: FONTE, color: C.tinta2, fontSize: 32 * e, marginTop: 12 }}>
          {cena.subtitulo}
        </p>
      )}
    </AbsoluteFill>
  );
}

/** O acento da marca como filete, quando o look pede.
 *
 *  Aqui o lime é FUNDO de um bloco sólido, não tinta: é o papel em que ele
 *  funciona nos dois temas do app, e a mesma regra vale num quadro de vídeo. */
function Filete() {
  return (
    <span
      style={{
        display: "block",
        width: 72,
        height: 6,
        borderRadius: 999,
        backgroundColor: C.acao,
        marginBottom: 20,
      }}
    />
  );
}

/** Escurece as bordas. Assenta o quadro e ajuda o texto a vencer. */
function Vinheta() {
  return (
    <AbsoluteFill
      style={{
        pointerEvents: "none",
        background:
          "radial-gradient(ellipse at center, rgba(0,0,0,0) 45%, rgba(0,0,0,0.55) 100%)",
      }}
    />
  );
}
