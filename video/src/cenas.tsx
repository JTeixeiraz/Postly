import React from "react";
import { AbsoluteFill, interpolate, useCurrentFrame } from "remotion";
import { C, FONTE, MONO, s } from "./tokens";
import { Brilho, Linha, Marca, saida, Titulo, useEntrada } from "./pecas";
import { Moldura, TelaCatalogo, TelaGrafo } from "./janela";
import { ClaudeCode, Geradores } from "./integracoes";

const PALCO: React.CSSProperties = {
  backgroundColor: C.fundo,
  padding: "0 128px",
  justifyContent: "center",
};

// ─────────────────────────────────────────────────────────── abertura

export const Abertura: React.FC = () => {
  const quadro = useCurrentFrame();
  const marca = useEntrada(0, 0, 22);
  const risco = interpolate(quadro, [10, 34], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: saida,
  });

  return (
    <AbsoluteFill style={PALCO}>
      <Brilho />
      <div style={{ display: "flex", alignItems: "center", gap: 26, ...marca }}>
        <Marca tamanho={104} />
        <span
          style={{
            fontFamily: FONTE,
            fontWeight: 690,
            fontSize: 122,
            letterSpacing: "-0.045em",
            color: C.tinta,
          }}
        >
          Postly
        </span>
      </div>

      {/* O fio cresce por scaleX: o mesmo gesto da trilha no app. */}
      <div
        style={{
          height: 2,
          width: 520,
          background: C.acao,
          borderRadius: 999,
          marginTop: 38,
          transform: `scaleX(${risco})`,
          transformOrigin: "left",
        }}
      />

      <div style={{ marginTop: 38 }}>
        <Linha atraso={26} tamanho={44} cor={C.tinta}>
          Um departamento de marketing que roda na sua máquina.
        </Linha>
      </div>

      <div style={{ marginTop: 30, ...useEntrada(40) }}>
        <span
          style={{
            fontFamily: MONO,
            fontSize: 26,
            color: C.acao,
            background: C.acaoLavado,
            padding: "10px 20px",
            borderRadius: 999,
          }}
        >
          open source · sem fins lucrativos
        </span>
      </div>
    </AbsoluteFill>
  );
};

// ─────────────────────────────────────────────────────────── o problema

export const Problema: React.FC = () => (
  <AbsoluteFill style={PALCO}>
    <Titulo largura={1500}>Ferramentas de marketing com IA cobram por mês.</Titulo>
    <div style={{ marginTop: 44 }}>
      <Linha atraso={16} tamanho={34} largura={1100}>
        Guardam suas campanhas num servidor alheio e escondem qual modelo
        escreveu o quê.
      </Linha>
    </div>
    <div style={{ marginTop: 26 }}>
      <Linha atraso={30} cor={C.acao} tamanho={40}>
        O Postly faz o contrário.
      </Linha>
    </div>
  </AbsoluteFill>
);

// ─────────────────────────────────────────────────────────── o revezamento

const CARGOS = [
  { cargo: "Diretor Geral", modelo: "qwen3:30b-a3b" },
  { cargo: "Gerente de Setor", modelo: "qwen3:30b-a3b" },
  { cargo: "Criador", modelo: "gemma3:4b" },
  { cargo: "Auditor", modelo: "qwen3:14b" },
];

export const Revezamento: React.FC = () => {
  const quadro = useCurrentFrame();
  const inicio = s(1.1);
  const porPosta = s(0.72);

  // A linha acesa acompanha a posta que já acendeu, meia posta à frente.
  const avanco = interpolate(
    quadro,
    [inicio, inicio + porPosta * (CARGOS.length - 1) + s(0.5)],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida }
  );

  return (
    <AbsoluteFill style={PALCO}>
      <Titulo tamanho={78}>Um modelo por vez</Titulo>
      <div style={{ marginTop: 30 }}>
        <Linha atraso={12} tamanho={30} largura={1180}>
          A cada troca de cargo o sistema mede a memória livre, sobe o modelo
          mais forte que couber, grava a conversa inteira e descarrega antes do
          próximo.
        </Linha>
      </div>

      <div style={{ position: "relative", marginTop: 92 }}>
        <div
          style={{
            position: "absolute",
            left: 11,
            right: 11,
            top: 11,
            height: 3,
            background: C.linha,
            borderRadius: 999,
          }}
        />
        <div
          style={{
            position: "absolute",
            left: 11,
            right: 11,
            top: 11,
            height: 3,
            background: C.acao,
            borderRadius: 999,
            transform: `scaleX(${avanco})`,
            transformOrigin: "left",
          }}
        />

        <div style={{ display: "flex", gap: 0 }}>
          {CARGOS.map((c, i) => {
            const t = interpolate(
              quadro,
              [inicio + i * porPosta, inicio + i * porPosta + s(0.4)],
              [0, 1],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp", easing: saida }
            );
            return (
              <div key={c.cargo} style={{ flex: 1, paddingRight: 40 }}>
                <div
                  style={{
                    width: 24,
                    height: 24,
                    borderRadius: "50%",
                    border: `3px solid ${C.acao}`,
                    background: C.acao,
                    opacity: 0.22 + t * 0.78,
                    transform: `scale(${0.7 + t * 0.3})`,
                    boxShadow: `0 0 0 ${t * 9}px rgba(201,242,39,0.10)`,
                  }}
                />
                <div
                  style={{
                    marginTop: 26,
                    fontFamily: FONTE,
                    fontWeight: 580,
                    fontSize: 34,
                    color: C.tinta,
                    opacity: 0.3 + t * 0.7,
                  }}
                >
                  {c.cargo}
                </div>
                <div
                  style={{
                    marginTop: 10,
                    fontFamily: MONO,
                    fontSize: 23,
                    color: C.acao,
                    opacity: t,
                  }}
                >
                  {c.modelo}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};

// ─────────────────────────────────────────────────────────── a inversão

/** Um dos dois modelos do comparativo. Definido fora da cena: componente
 *  declarado dentro de outro é recriado a cada quadro e perde o estado. */
const Lado: React.FC<{
  nome: string;
  tipo: string;
  disco: string;
  valor: string;
  pct: number;
  atraso: number;
  vence?: boolean;
}> = ({ nome, tipo, disco, valor, pct, atraso, vence }) => {
  const quadro = useCurrentFrame();
  const e = useEntrada(atraso - s(0.3));
  const largura = interpolate(quadro, [atraso, atraso + s(0.9)], [0, pct], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: saida,
  });
  return (
      <div
        style={{
          ...e,
          flex: 1,
          padding: 44,
          borderRadius: 22,
          border: `1px solid ${vence ? C.acao : C.linha}`,
          background: vence ? C.acaoLavado : C.cartao,
        }}
      >
        <div style={{ fontFamily: MONO, fontSize: 38, color: C.tinta }}>{nome}</div>
        <div style={{ fontFamily: FONTE, fontSize: 26, color: C.tinta2, marginTop: 12 }}>
          {tipo}
        </div>
        <div style={{ fontFamily: FONTE, fontSize: 23, color: C.tinta3, marginTop: 6 }}>
          {disco}
        </div>
        <div
          style={{
            height: 12,
            borderRadius: 999,
            background: C.afundado,
            marginTop: 32,
            overflow: "hidden",
          }}
        >
          <div
            style={{
              height: "100%",
              width: `${largura}%`,
              background: vence ? C.acao : C.tinta3,
              borderRadius: 999,
            }}
          />
        </div>
        <div
          style={{
            fontFamily: MONO,
            fontSize: 46,
            color: vence ? C.acao : C.tinta,
            marginTop: 20,
          }}
        >
          {valor}
        </div>
    </div>
  );
};

export const Inversao: React.FC = () => {
  return (
    <AbsoluteFill style={PALCO}>
      <Titulo tamanho={68} largura={1400}>
        O catálogo ranqueia por velocidade,
        <br />
        não por tamanho.
      </Titulo>

      <div style={{ display: "flex", gap: 40, marginTop: 62 }}>
        <Lado
          nome="qwen3:14b"
          tipo="denso · 14B ativos"
          disco="9,3 GB em disco"
          valor="0,6 tok/s"
          pct={11}
          atraso={s(1.2)}
        />
        <Lado
          nome="qwen3:30b-a3b"
          tipo="MoE · 3B ativos de 30B"
          disco="19 GB em disco"
          valor="5,7 tok/s"
          pct={100}
          atraso={s(1.5)}
          vence
        />
      </div>

      <div style={{ marginTop: 42 }}>
        <Linha atraso={s(2.4)} tamanho={26} cor={C.tinta3} largura={1200}>
          Medido sem GPU. O modelo que ocupa o dobro de memória gera quase dez
          vezes mais rápido, porque só os especialistas ativos passam pela CPU.
        </Linha>
      </div>
    </AbsoluteFill>
  );
};

// ─────────────────────────────────────────────────────────── as telas

export const Telas: React.FC = () => {
  const quadro = useCurrentFrame();
  // Duas telas, desenhadas e animadas — não capturadas. O catálogo se monta
  // linha a linha e o grafo se abre do centro: é o que a interface faz quando
  // alguém a usa, e é isso que uma apresentação precisa mostrar.
  const troca = s(3.4);
  const noGrafo = quadro >= troca;
  const op = noGrafo
    ? interpolate(quadro, [troca, troca + s(0.32)], [0, 1], { extrapolateLeft: "clamp", extrapolateRight: "clamp" })
    : interpolate(quadro, [troca - s(0.32), troca], [1, 0], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });

  return (
    <AbsoluteFill style={{ ...PALCO, justifyContent: "center", padding: "0 96px" }}>
      <Titulo tamanho={54}>O que você abre</Titulo>
      <div style={{ opacity: op, marginTop: 34 }}>
        <Moldura aba={noGrafo ? "Cérebro" : "Modelos"}>
          {noGrafo ? <TelaGrafo atraso={troca} /> : <TelaCatalogo />}
        </Moldura>
      </div>
    </AbsoluteFill>
  );
};

// ─────────────────────────────────────────────────────────── privacidade

/** Um item da lista. Componente próprio e não um trecho dentro do `map`:
 *  hook em laço quebra a regra dos hooks no instante em que a lista mudar de
 *  tamanho. */
const ItemQueFica: React.FC<{ texto: string; atraso: number }> = ({ texto, atraso }) => {
  const e = useEntrada(atraso, 16);
  return (
    <div style={{ ...e, display: "flex", alignItems: "center", gap: 22 }}>
      <span
        style={{ width: 12, height: 12, borderRadius: "50%", background: C.acao, flex: "none" }}
      />
      <span style={{ fontFamily: FONTE, fontSize: 37, color: C.tinta }}>{texto}</span>
    </div>
  );
};

export const Privacidade: React.FC = () => {
  const itens = [
    "Os modelos e tudo que eles escrevem",
    "O grafo de contexto e as campanhas",
    "As transcrições, turno por turno",
    "As credenciais, cifradas no disco",
  ];

  return (
    <AbsoluteFill style={PALCO}>
      <Titulo tamanho={72}>Fica tudo na sua máquina</Titulo>

      <div style={{ marginTop: 54, display: "grid", gap: 22 }}>
        {itens.map((t, i) => (
          <ItemQueFica key={t} texto={t} atraso={s(0.7) + i * s(0.28)} />
        ))}
      </div>

      <div style={{ marginTop: 52 }}>
        <Linha atraso={s(2.1)} tamanho={27} cor={C.tinta3} largura={1080}>
          Sai da máquina só a geração de imagem e o navegador que publica nas
          suas próprias contas. Não há telemetria, servidor do projeto, nem
          conta para criar.
        </Linha>
      </div>
    </AbsoluteFill>
  );
};

// ─────────────────────────────────────────────────────────── fecho

export const Fecho: React.FC = () => {
  const marca = useEntrada(0, 0, 22);
  const comando = useEntrada(s(0.9));
  const link = useEntrada(s(1.6));

  return (
    <AbsoluteFill style={{ ...PALCO, alignItems: "center", textAlign: "center" }}>
      <Brilho x="50%" y="52%" raio={1200} />

      <div style={{ display: "flex", alignItems: "center", gap: 22, ...marca }}>
        <Marca tamanho={68} />
        <span
          style={{
            fontFamily: FONTE,
            fontWeight: 690,
            fontSize: 88,
            letterSpacing: "-0.04em",
            color: C.tinta,
          }}
        >
          Postly
        </span>
      </div>

      <div style={{ marginTop: 30, ...useEntrada(s(0.4)) }}>
        <span style={{ fontFamily: FONTE, fontSize: 36, color: C.tinta2 }}>
          Gratuito, MIT, sem fins lucrativos. Não há versão paga.
        </span>
      </div>

      <div
        style={{
          ...comando,
          marginTop: 52,
          background: C.afundado,
          border: `1px solid ${C.linha}`,
          borderRadius: 16,
          padding: "24px 34px",
          fontFamily: MONO,
          fontSize: 19,
          color: C.tinta,
          display: "flex",
          gap: 16,
        }}
      >
        <span style={{ color: C.acao }}>$</span>
        <span>curl -fsSL https://raw.githubusercontent.com/JTeixeiraz/Postly/main/scripts/instalar.sh | bash</span>
      </div>

      <div
        style={{
          ...link,
          marginTop: 40,
          fontFamily: MONO,
          fontSize: 30,
          color: C.acao,
        }}
      >
        github.com/JTeixeiraz/Postly
      </div>
    </AbsoluteFill>
  );
};


/* ══ integrações ═════════════════════════════════════════════════════════ */

export const Claude: React.FC = () => (
  <AbsoluteFill style={{ ...PALCO, justifyContent: "center", padding: "0 88px" }}>
    <Titulo tamanho={62}>Ou os cargos rodam no seu Claude Code</Titulo>
    <div style={{ marginTop: 52 }}>
      <ClaudeCode />
    </div>
  </AbsoluteFill>
);

export const Arte: React.FC = () => (
  <AbsoluteFill style={{ ...PALCO, justifyContent: "center", alignItems: "center" }}>
    <Titulo tamanho={62}>E a arte, em quem você escolher</Titulo>
    <div style={{ marginTop: 56 }}>
      <Geradores />
    </div>
  </AbsoluteFill>
);
