import React from "react";
import { interpolate, useCurrentFrame } from "remotion";
import { C, FONTE, MONO, s } from "./tokens";
import { saida } from "./pecas";

/** As duas cenas de integração.
 *
 *  Vêm depois do tour de telas porque respondem à pergunta que ele levanta:
 *  "e se eu não quiser rodar tudo local?". A primeira mostra que os cargos
 *  podem ser executados pelo Claude Code que a pessoa já assina; a segunda,
 *  que a arte não está presa a um fornecedor. */

/* ══ Claude Code ═════════════════════════════════════════════════════════ */

const CARGOS_CLAUDE = [
  { cargo: "Diretor Geral", modelo: "Opus 5", nota: "decide a linha" },
  { cargo: "Gerente de Setor", modelo: "Opus 5", nota: "define o setor" },
  { cargo: "Auditor", modelo: "Sonnet 5", nota: "julga a peça" },
  { cargo: "Criador", modelo: "Haiku 4.5", nota: "executa" },
];

/** O “A” da Anthropic, geométrico e próprio — não a marca registrada. */
const GlifoAnthropic: React.FC<{ tamanho?: number }> = ({ tamanho = 34 }) => (
  <svg width={tamanho} height={tamanho} viewBox="0 0 32 32">
    <path
      d="M16 3.5 L27.5 28 H21.4 L19.2 22.6 H12.8 L10.6 28 H4.5 Z M16 11.6 L14 17.2 H18 Z"
      fill={C.tinta}
    />
  </svg>
);

export const ClaudeCode: React.FC = () => {
  const f = useCurrentFrame();
  const entra = (atraso: number) =>
    interpolate(f, [atraso, atraso + s(0.5)], [0, 1], {
      extrapolateLeft: "clamp",
      extrapolateRight: "clamp",
      easing: saida,
    });

  return (
    <div style={{ display: "grid", gap: 46 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 18, opacity: entra(0) }}>
        <GlifoAnthropic tamanho={58} />
        <span style={{ fontFamily: FONTE, fontSize: 56, fontWeight: 620, color: C.tinta, letterSpacing: "-0.03em" }}>
          Claude Code
        </span>
        <span
          style={{
            fontFamily: FONTE, fontSize: 24, fontWeight: 520,
            padding: "12px 22px", borderRadius: 999,
            background: C.acaoLavado, color: C.acao, marginLeft: 8,
          }}
        >
          sua assinatura, sem chave de API
        </span>
      </div>

      {/* O mesmo organograma, outro executor: é essa a ideia que a cena
          precisa passar — o nível do cargo continua mandando. */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 20 }}>
        {CARGOS_CLAUDE.map((c, i) => {
          const e = entra(s(0.35) + i * s(0.16));
          return (
            <div
              key={c.cargo}
              style={{
                background: C.cartao, borderRadius: 22, padding: "38px 32px",
                display: "grid", gap: 12, alignContent: "start",
                opacity: e, transform: `translateY(${(1 - e) * 18}px)`,
              }}
            >
              <span style={{ fontFamily: FONTE, fontSize: 23, color: C.tinta3 }}>{c.cargo}</span>
              <span style={{ fontFamily: FONTE, fontSize: 36, fontWeight: 580, color: C.tinta, letterSpacing: "-0.02em" }}>
                {c.modelo}
              </span>
              <span style={{ fontFamily: MONO, fontSize: 20, color: C.acao }}>{c.nota}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
};

/* ══ geradores de imagem ═════════════════════════════════════════════════ */

/* Glifos geométricos próprios, não as marcas registradas de cada laboratório:
   reproduzir logo de terceiro num projeto que vai para o mundo é problema de
   licença. A forma distingue; a cor é a da casa, em dose pequena. */
const MARCAS: { nome: string; cor: string; glifo: React.ReactNode }[] = [
  {
    nome: "Gemini", cor: "#4C8BF5",
    glifo: <path d="M16 3 C17 11 21 15 29 16 C21 17 17 21 16 29 C15 21 11 17 3 16 C11 15 15 11 16 3 Z" />,
  },
  {
    nome: "OpenAI", cor: "#10A37F",
    glifo: <path d="M16 4 L26.4 10 V22 L16 28 L5.6 22 V10 Z M16 9.2 L10 12.6 V19.4 L16 22.8 L22 19.4 V12.6 Z" />,
  },
  {
    nome: "FLUX", cor: "#E85D3D",
    glifo: <path d="M6 6 H26 L20 13 H6 Z M6 17 H20 L14 24 H6 Z" />,
  },
  {
    nome: "Stability", cor: "#8B5CF6",
    glifo: <path d="M16 3 L28 16 L16 29 L4 16 Z M16 10.4 L10.4 16 L16 21.6 L21.6 16 Z" />,
  },
  {
    nome: "Higgsfield", cor: "#D9D9D9",
    glifo: <path d="M16 3 L20.6 9.6 L28.4 8 L26.8 15.8 L31 21.5 L23.6 24 L21.5 31 L16 26.2 L10.5 31 L8.4 24 L1 21.5 L5.2 15.8 L3.6 8 L11.4 9.6 Z" />,
  },
];

export const Geradores: React.FC = () => {
  const f = useCurrentFrame();
  return (
    <div style={{ display: "grid", gap: 52, justifyItems: "center" }}>
      <div style={{ display: "flex", gap: 34 }}>
        {MARCAS.map((m, i) => {
          const e = interpolate(f, [s(0.2) + i * s(0.14), s(0.72) + i * s(0.14)], [0, 1], {
            extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida,
          });
          return (
            <div
              key={m.nome}
              style={{
                background: C.cartao, borderRadius: 22,
                width: 250, padding: "44px 26px",
                display: "grid", gap: 22, justifyItems: "center",
                opacity: e, transform: `translateY(${(1 - e) * 22}px) scale(${0.94 + e * 0.06})`,
              }}
            >
              <svg width={72} height={72} viewBox="0 0 32 32" fill={m.cor}>
                {m.glifo}
              </svg>
              <span style={{ fontFamily: FONTE, fontSize: 26, fontWeight: 560, color: C.tinta }}>
                {m.nome}
              </span>
            </div>
          );
        })}
      </div>

      <span
        style={{
          fontFamily: FONTE, fontSize: 28, color: C.tinta3,
          opacity: interpolate(f, [s(1.3), s(1.9)], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" }),
        }}
      >
        A chave é sua, e fica cifrada no seu disco.
      </span>
    </div>
  );
};
