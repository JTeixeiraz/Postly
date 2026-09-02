import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { ProgressoRender, VideoPronto } from "../types";

/** O monitor: onde o vídeo toca.
 *
 *  O PREVIEW É O ARQUIVO RENDERIZADO, e não uma reconstrução em HTML das cenas.
 *  Redesenhar as cenas em CSS daria uma prévia antes do render, mas custaria
 *  duas fontes de verdade para a mesma coisa: o dia em que a `ken_burns` do
 *  Remotion mudasse e a do preview não, a pessoa aprovaria um vídeo e receberia
 *  outro. Um monitor que mostra exatamente o que foi entregue nunca mente.
 *
 *  Antes do primeiro render ele fica vazio, com o motivo escrito. Uma tela
 *  preta sem explicação pareceria defeito. */
export default function Monitor({
  video,
  segundo,
  aoAndar,
  render,
  buscarPara,
  jaRodou,
}: {
  video: VideoPronto | null;
  segundo: number;
  aoAndar: (s: number) => void;
  render: ProgressoRender | null;
  /** Instante para onde o monitor deve pular, quando a linha do tempo pede. */
  buscarPara: number | null;
  /** Já houve uma rodada neste projeto?
   *
   *  Sem isto o vazio diria sempre "preencha o briefing e gere" — inclusive depois de uma
   *  rodada em que o render falhou, mandando a pessoa refazer o passo que ela já fez. */
  jaRodou: boolean;
}) {
  const { d } = useIdioma();
  const ref = useRef<HTMLVideoElement>(null);
  // O monitor pode não conseguir tocar, e o motivo não é o arquivo: a WebView
  // do Linux (WebKitGTK) decodifica vídeo pelo GStreamer, e o H.264 vem do
  // pacote `gst-libav`, que não está em toda distribuição. MEDIDO nesta
  // máquina: `avdec_h264` ausente, e o monitor fica um retângulo preto.
  //
  // Sem este estado, esse retângulo pareceria um defeito do Postly. Com ele, a
  // tela diz o que aconteceu e oferece a saída que sempre funciona — abrir no
  // player do sistema, que tem os codecs que a WebView não tem.
  const [semCodec, setSemCodec] = useState(false);

  // A busca vem de fora (clique numa cena, clique na régua) e o vídeo é quem
  // manda no tempo depois disso. Sem este efeito o cabeçote se moveria e a
  // imagem não — a linha do tempo viraria enfeite.
  useEffect(() => {
    if (buscarPara === null || !ref.current) return;
    ref.current.currentTime = buscarPara;
  }, [buscarPara]);

  // Um arquivo novo merece uma tentativa nova: o vídeo anterior pode ter
  // falhado por um motivo que não vale para este.
  useEffect(() => setSemCodec(false), [video?.arquivo]);

  if (!video) {
    return (
      <div className="monitor monitor--vazio">
        {render ? (
          <>
            <span className="tag" data-tone="live">
              <span className="tag__dot" />
              {render.fase === "empacotando" ? d.video.bundling : d.video.rendering}
            </span>
            <div className="provisao__barra">
              <span style={{ width: `${Math.round(render.percent * 100)}%` }} />
            </div>
          </>
        ) : (
          <p className="hint">{jaRodou ? d.monitor.semVideo : d.monitor.empty}</p>
        )}
      </div>
    );
  }

  return (
    <div className="monitor">
      {semCodec ? (
        <div className="monitor__semcodec">
          <p>{d.monitor.noCodec}</p>
          <p className="hint">{d.monitor.noCodecFix}</p>
          <button className="btn btn--key" onClick={() => void api.abrirNoSistema(video.arquivo)}>
            {d.monitor.openOutside}
          </button>
        </div>
      ) : (
        <video
          ref={ref}
          controls
          src={convertFileSrc(video.arquivo)}
          onTimeUpdate={(e) => aoAndar(e.currentTarget.currentTime)}
          // `MEDIA_ERR_SRC_NOT_SUPPORTED` (4) é exatamente o caso do codec
          // faltando. Os outros códigos são rede e decodificação, e cair na
          // mesma tela para todos seria acusar o codec por um erro que pode
          // ser outro — mas o texto cobre os dois, então a distinção não
          // mudaria o que a pessoa faz.
          onError={() => setSemCodec(true)}
        />
      )}
      <div className="monitor__pe">
        <span className="num">
          {relogio(segundo)} / {relogio(video.duracao_s)}
        </span>
      </div>
    </div>
  );
}

/** Segundos em `m:ss,d`.
 *
 *  Com décimo de segundo de propósito: a nota que a pessoa escreve carrega o
 *  instante, e um relógio arredondado ao segundo apontaria para um lugar até
 *  meio segundo longe do que ela está vendo. */
function relogio(s: number) {
  const m = Math.floor(s / 60);
  const r = s - m * 60;
  return `${m}:${r.toFixed(1).padStart(4, "0")}`;
}
