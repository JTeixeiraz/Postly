import { useMemo } from "react";
import { useIdioma } from "../i18n";
import type { CenaVideo, RoteiroVideo } from "../types";

/** A linha do tempo: régua, trilhas e cabeçote.
 *
 *  PARECE UM EDITOR E NÃO É UM. Não há alça para arrastar, não há corte, não há
 *  keyframe. A linha existe para a pessoa **ler** o que o Motion Designer
 *  montou e **apontar** onde está errado — clicar numa cena leva o vídeo até
 *  ela, e a nota que ela escrever volta para o cargo refazer.
 *
 *  A forma de editor é deliberada mesmo sem a função: uma lista de cenas em
 *  texto não mostra ritmo. Blocos proporcionais à duração mostram, num relance,
 *  que a cena 3 tem o dobro da 2 — que é exatamente o tipo de coisa que a
 *  pessoa consegue julgar e o modelo erra. */
export default function Linha({
  roteiro,
  selecionada,
  aoSelecionar,
  segundo,
  aoBuscar,
  narracao,
  temTrilha,
}: {
  roteiro: RoteiroVideo;
  /** Índice base 1, como a tela numera. `null` = nenhuma. */
  selecionada: number | null;
  aoSelecionar: (i: number) => void;
  /** Onde o cabeçote está, em segundos. */
  segundo: number;
  aoBuscar: (s: number) => void;
  narracao: number;
  temTrilha: boolean;
}) {
  const { d } = useIdioma();

  const { total, inicios } = useMemo(() => {
    let acc = 0;
    const inicios = roteiro.cenas.map((c) => {
      const de = acc;
      acc += c.dur_s;
      return de;
    });
    return { total: acc || 1, inicios };
  }, [roteiro]);

  // Marcas de segundo inteiro. Acima de ~24 marcas elas viram um pente
  // ilegível, então o passo cresce — 1s, 2s, 5s, 10s.
  const passo = [1, 2, 5, 10].find((p) => total / p <= 24) ?? 30;
  const marcas: number[] = [];
  for (let t = 0; t <= total; t += passo) marcas.push(t);

  const pct = (s: number) => `${(s / total) * 100}%`;

  return (
    <div className="linha">
      {/* Clicar na régua move o cabeçote: é o gesto que a pessoa já conhece de
          qualquer player, e sem ele a única forma de chegar num instante seria
          selecionar a cena inteira. */}
      <div
        className="linha__regua"
        onClick={(e) => {
          const r = e.currentTarget.getBoundingClientRect();
          aoBuscar(((e.clientX - r.left) / r.width) * total);
        }}
      >
        {marcas.map((t) => (
          <span className="linha__marca" key={t} style={{ left: pct(t) }}>
            <i />
            <em className="num">{t}s</em>
          </span>
        ))}
      </div>

      <div className="linha__trilhas">
        <Trilha rotulo={d.linha.scenes}>
          {roteiro.cenas.map((c, i) => (
            <Bloco
              key={i}
              cena={c}
              numero={i + 1}
              esquerda={pct(inicios[i])}
              largura={pct(c.dur_s)}
              ativa={selecionada === i + 1}
              tocando={segundo >= inicios[i] && segundo < inicios[i] + c.dur_s}
              aoClicar={() => {
                aoSelecionar(i + 1);
                aoBuscar(inicios[i]);
              }}
            />
          ))}
        </Trilha>

        {/* As trilhas de áudio aparecem só quando há áudio. Uma trilha vazia
            desenhada mesmo assim sugeriria um lugar onde soltar arquivo, e não
            há: o áudio entra pela aba de assets. */}
        {narracao > 0 && (
          <Trilha rotulo={d.linha.voice}>
            <span className="linha__onda" data-tipo="voz" style={{ width: "100%" }}>
              {narracao} {narracao === 1 ? d.linha.file : d.linha.files}
            </span>
          </Trilha>
        )}
        {temTrilha && (
          <Trilha rotulo={d.linha.track}>
            <span className="linha__onda" data-tipo="trilha" style={{ width: "100%" }}>
              {roteiro.trilha}
            </span>
          </Trilha>
        )}

        <span className="linha__cabecote" style={{ left: pct(Math.min(segundo, total)) }} />
      </div>
    </div>
  );
}

function Trilha({ rotulo, children }: { rotulo: string; children: React.ReactNode }) {
  return (
    <div className="linha__trilha">
      <span className="linha__rot">{rotulo}</span>
      <div className="linha__pista">{children}</div>
    </div>
  );
}

function Bloco({
  cena,
  numero,
  esquerda,
  largura,
  ativa,
  tocando,
  aoClicar,
}: {
  cena: CenaVideo;
  numero: number;
  esquerda: string;
  largura: string;
  ativa: boolean;
  tocando: boolean;
  aoClicar: () => void;
}) {
  return (
    <button
      className="linha__bloco"
      style={{ left: esquerda, width: largura }}
      data-ativa={ativa}
      data-tocando={tocando}
      onClick={aoClicar}
      title={`${numero}. ${cena.titulo || cena.tipo} · ${cena.dur_s.toFixed(1)}s`}
    >
      <span className="linha__bloco-n num">{numero}</span>
      <span className="linha__bloco-t">{cena.titulo || cena.tipo}</span>
    </button>
  );
}
