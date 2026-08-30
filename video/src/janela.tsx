import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { C, FONTE, MONO, s } from "./tokens";
import { Marca, saida } from "./pecas";

/** A janela do aplicativo, desenhada.
 *
 *  Não é captura de tela: é o mesmo desenho, remontado com os tokens do
 *  produto e animado por quadro. Uma imagem estática mostraria a mesma coisa
 *  parada, e num vídeo de apresentação o que se quer ver é a interface
 *  trabalhando — a lista chegando, a trilha acendendo.
 *
 *  Toda animação aqui é função do frame, nunca de transição CSS: a renderização
 *  é determinística e um `transition` não existiria no arquivo final. */

const ABAS = ["Modelos", "Campanha", "Cérebro"] as const;

export const Moldura: React.FC<{
  aba: (typeof ABAS)[number];
  children: React.ReactNode;
  atraso?: number;
}> = ({ aba, children, atraso = 0 }) => {
  const f = useCurrentFrame() - atraso;
  const entra = interpolate(f, [0, s(0.7)], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: saida,
  });

  return (
    <div
      style={{
        // Papel branco em volta do cartão escuro: é assim que o Postly abre.
        background: "#FDFDFD",
        borderRadius: 26,
        padding: "0 14px 14px",
        boxShadow: "0 8px 24px rgba(0,0,0,.34), 0 40px 90px rgba(0,0,0,.42)",
        opacity: entra,
        transform: `translateY(${(1 - entra) * 26}px) scale(${0.985 + entra * 0.015})`,
      }}
    >
      <header style={{ display: "flex", alignItems: "center", gap: 22, padding: "14px 10px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 9, color: "#3A3F45" }}>
          <Marca tamanho={22} cor="#3A3F45" />
          <span style={{ fontFamily: FONTE, fontSize: 18, fontWeight: 620, letterSpacing: "-0.025em" }}>
            postly
          </span>
        </div>

        <nav style={{ display: "flex", gap: 4, background: "#F1F2F3", borderRadius: 999, padding: 4 }}>
          {ABAS.map((a) => {
            const ativa = a === aba;
            return (
              <span
                key={a}
                style={{
                  fontFamily: FONTE, fontSize: 15, fontWeight: ativa ? 560 : 480,
                  padding: "8px 16px", borderRadius: 999,
                  background: ativa ? C.acao : "transparent",
                  color: ativa ? C.acaoTinta : "#6B717A",
                }}
              >
                {a}
              </span>
            );
          })}
        </nav>

        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 12 }}>
          <div style={{ width: 54, height: 7, borderRadius: 999, background: "#E2E4E6", overflow: "hidden" }}>
            <div style={{ width: "34%", height: "100%", background: "#9AA1AA" }} />
          </div>
          <span style={{ fontFamily: MONO, fontSize: 13, color: "#6B717A" }}>21,5 GB livre</span>
        </div>
      </header>

      <div
        style={{
          background: C.fundo, borderRadius: 20, padding: "26px 30px 30px",
          height: 520, overflow: "hidden",
        }}
      >
        {children}
      </div>
    </div>
  );
};

/* ══ tela: catálogo ═══════════════════════════════════════════════════════ */

const CATALOGO = [
  { marca: "#7C6BD8", nome: "Qwen 3 30B-A3B (MoE)", tag: "qwen3:30b-a3b", peso: "20,1 GB", tps: "5,7", rapido: true, selo: "baixado" },
  { marca: "#10A37F", nome: "GPT-OSS 20B (MoE)", tag: "gpt-oss:20b", peso: "15,4 GB", tps: "4,9", rapido: true },
  { marca: "#4C8BF5", nome: "Gemma 3 4B", tag: "gemma3:4b", peso: "3,9 GB", tps: "12,4", rapido: true, selo: "baixado" },
  { marca: "#5B7FE8", nome: "DeepSeek R1 14B", tag: "deepseek-r1:14b", peso: "9,8 GB", tps: "1,1" },
  { marca: "#EC650E", nome: "Mistral Small 24B", tag: "mistral-small:24b", peso: "15,1 GB", tps: "0,7" },
];

export const TelaCatalogo: React.FC<{ atraso?: number }> = ({ atraso = 0 }) => {
  const f = useCurrentFrame() - atraso;
  return (
    <>
      <div style={{ fontFamily: FONTE, fontSize: 25, fontWeight: 600, color: C.tinta, letterSpacing: "-0.02em" }}>
        O que roda aqui
      </div>
      <div style={{ fontFamily: FONTE, fontSize: 16, color: C.tinta3, marginTop: 6, marginBottom: 26 }}>
        Você não escolhe. A cada cargo, sobe o modelo mais forte que couber.
      </div>

      {CATALOGO.map((m, i) => {
        // As linhas chegam em cascata: a lista se montando diz "isto é uma
        // ferramenta viva" melhor que a lista pronta.
        const e = interpolate(f, [s(0.2) + i * s(0.14), s(0.7) + i * s(0.14)], [0, 1], {
          extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida,
        });
        return (
          <div
            key={m.tag}
            style={{
              display: "flex", alignItems: "center", gap: 16,
              padding: "17px 0", borderBottom: `1px solid ${C.linha}`,
              opacity: e, transform: `translateX(${(1 - e) * 22}px)`,
            }}
          >
            <div style={{
              width: 26, height: 26, borderRadius: 7, background: m.marca,
              flex: "none", opacity: 0.9,
            }} />
            <div style={{ flex: 1 }}>
              <div style={{ display: "flex", alignItems: "baseline", gap: 11 }}>
                <span style={{ fontFamily: FONTE, fontSize: 19, fontWeight: 540, color: C.tinta }}>{m.nome}</span>
                <span style={{ fontFamily: MONO, fontSize: 13, color: C.tinta3 }}>{m.tag}</span>
                {m.selo && (
                  <span style={{
                    fontFamily: FONTE, fontSize: 12.5, fontWeight: 500,
                    color: "#7BD97B", background: "rgba(60,160,90,.18)",
                    padding: "4px 10px", borderRadius: 999,
                  }}>{m.selo}</span>
                )}
              </div>
            </div>
            <div style={{ textAlign: "right" }}>
              <div style={{ fontFamily: FONTE, fontSize: 18, fontWeight: 560, color: C.tinta }}>{m.peso}</div>
              <div style={{ fontFamily: MONO, fontSize: 13, color: m.rapido ? "#8FD98F" : C.tinta3 }}>
                ≈ {m.tps} tok/s
              </div>
            </div>
          </div>
        );
      })}
    </>
  );
};

/* ══ tela: grafo ══════════════════════════════════════════════════════════ */

// Posições em fração da área de desenho. Ficam longe das bordas de propósito:
// o rótulo cresce para a direita do node e um node a 0,9 perde o nome.
const NOS = [
  { id: "publico_alvo", x: 0.44, y: 0.50, forte: true },
  { id: "tom_de_voz", x: 0.70, y: 0.24, peso: 0.94 },
  { id: "instagram", x: 0.22, y: 0.20, peso: 0.81 },
  { id: "objecao_preco", x: 0.18, y: 0.78, peso: 0.63 },
  { id: "prova_social", x: 0.66, y: 0.80, peso: 0.47 },
  { id: "produto", x: 0.80, y: 0.54 },
  { id: "linkedin", x: 0.10, y: 0.48 },
];

export const TelaGrafo: React.FC<{ atraso?: number }> = ({ atraso = 0 }) => {
  const f = useCurrentFrame() - atraso;
  // O viewBox acompanha a proporção do espaço real: com `width: 100%` e altura
  // automática, um viewBox mais alto que a área faz o SVG crescer além do
  // cartão e os nodes de baixo somem no corte.
  const L = 1640, A = 470;
  const centro = NOS[0];

  return (
    <>
      <div style={{ fontFamily: FONTE, fontSize: 25, fontWeight: 600, color: C.tinta, letterSpacing: "-0.02em" }}>
        Cérebro
      </div>
      <div style={{ fontFamily: FONTE, fontSize: 16, color: C.tinta3, marginTop: 6, marginBottom: 10 }}>
        O contexto que todos os cargos compartilham, em grafo ponderado.
      </div>

      <svg width="100%" viewBox={`0 0 ${L} ${A}`} style={{ display: "block" }}>
        {NOS.slice(1).map((n, i) => {
          // A aresta se desenha do centro para fora, na ordem do peso: é a
          // mesma leitura que o agente recebe ao consultar.
          const e = interpolate(f, [s(0.3) + i * s(0.12), s(0.9) + i * s(0.12)], [0, 1], {
            extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida,
          });
          const x1 = centro.x * L, y1 = centro.y * A;
          const x2 = x1 + (n.x * L - x1) * e, y2 = y1 + (n.y * A - y1) * e;
          const forte = (n.peso ?? 0.3) > 0.6;
          return (
            <g key={n.id}>
              <line
                x1={x1} y1={y1} x2={x2} y2={y2}
                stroke={forte ? C.acao : C.linha}
                strokeWidth={forte ? 3.4 : 2}
                strokeOpacity={forte ? 0.85 : 0.7}
                strokeLinecap="round"
              />
              {n.peso && e > 0.85 && (
                <text
                  x={(x1 + n.x * L) / 2} y={(y1 + n.y * A) / 2 - 13}
                  fill={C.tinta3} fontFamily={MONO} fontSize={22} textAnchor="middle"
                  opacity={interpolate(e, [0.85, 1], [0, 1])}
                >
                  {n.peso.toFixed(2).replace(".", ",")}
                </text>
              )}
            </g>
          );
        })}

        {NOS.map((n, i) => {
          const e = interpolate(f, [s(0.15) + i * s(0.1), s(0.6) + i * s(0.1)], [0, 1], {
            extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida,
          });
          const r = (n.forte ? 17 : 11) * e;
          return (
            <g key={n.id} opacity={e}>
              <circle cx={n.x * L} cy={n.y * A} r={r} fill={n.forte ? C.acao : C.tinta3} />
              <text
                x={n.x * L + r + 13} y={n.y * A + 9}
                fill={n.forte ? C.tinta : C.tinta2}
                fontFamily={MONO} fontSize={26}
              >
                {n.id}
              </text>
            </g>
          );
        })}
      </svg>
    </>
  );
};
