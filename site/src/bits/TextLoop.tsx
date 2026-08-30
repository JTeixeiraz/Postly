import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { gsap } from "gsap";
import "./TextLoop.css";

const L = 1200;

/** Altura da caixa de desenho.
 *
 *  Duas parcelas: a letra (ascendente, descendente e a folga do acento — a
 *  frase tem "MÁQUINA") e a onda, que desloca o baseline. Uma Bézier
 *  quadrática não alcança o próprio ponto de controle, então a onda ocupa
 *  cerca de metade da amplitude pedida. */
const alturaDaCaixa = (tamanho: number, amplitude: number) =>
  Math.ceil(tamanho * 2 + amplitude);

/** Texto correndo por uma curva.
 *
 *  De reactbits.dev, com a faixa colorida removida — ela competiria com o
 *  único acento do sistema — e com o laço refeito.
 *
 *  O original alterna dois `<textPath>`, um deles em `startOffset` negativo.
 *  Fora do intervalo do caminho o navegador não tem curva para seguir e
 *  projeta as letras onde quiser: aqui elas subiam 25 unidades acima da caixa
 *  e apareciam decapitadas na borda de cima. Este usa um `<textPath>` só, com
 *  texto repetido além do necessário e deslocamento sempre positivo, indo da
 *  largura de uma repetição de volta a zero. O emendo é invisível porque o
 *  que entra é uma cópia idêntica do que saiu. */
export default function TextLoop({
  texto = "código aberto",
  separador = "✦",
  velocidade = 46,
  amplitude = 34,
  tamanho = 34,
}: {
  texto?: string;
  separador?: string;
  velocidade?: number;
  amplitude?: number;
  tamanho?: number;
}) {
  const pathRef = useRef<SVGPathElement>(null);
  const medidaRef = useRef<SVGTextElement>(null);
  const correnteRef = useRef<SVGTextPathElement>(null);
  const [m, setM] = useState({ comprimento: 0, unidade: 0, repeticoes: 2 });

  const id = `loop-${useId().replace(/:/g, "")}`;
  const A = alturaDaCaixa(tamanho, amplitude);
  const CY = A / 2;

  /* O caminho começa MUITO antes da área visível.
   *
   *  O deslocamento é a distância ao longo do caminho onde o texto começa, e
   *  ele varre uma repetição inteira a cada volta. Se o caminho começasse na
   *  borda esquerda, no meio do ciclo o texto estaria começando lá pelo meio
   *  da faixa e a metade esquerda ficaria vazia. Com a entrada bem atrás,
   *  qualquer deslocamento do ciclo ainda deixa letras cobrindo x = 0. */
  const d = useMemo(() => {
    const PASSO = 320;
    const inicio = -(Math.ceil(1600 / PASSO) * PASSO);
    const partes = [`M ${inicio} ${CY}`, `Q ${inicio + PASSO / 2} ${CY - amplitude} ${inicio + PASSO} ${CY}`];
    for (let x = inicio + PASSO; x < L + PASSO; x += PASSO) {
      partes.push(`T ${x + PASSO} ${CY}`);
    }
    return partes.join(" ");
  }, [amplitude, CY]);

  const unidade = useMemo(
    () => `${texto.toUpperCase()} ${separador} `,
    [texto, separador]
  );

  useLayoutEffect(() => {
    const p = pathRef.current;
    const med = medidaRef.current;
    if (!p || !med) return;
    const medir = () => {
      try {
        const comprimento = p.getTotalLength();
        const larguraUnidade = med.getComputedTextLength();
        if (!comprimento || !larguraUnidade) return;
        // Uma repetição a mais do que cabe: no fim do ciclo o deslocamento
        // chega a zero e a cauda precisa continuar cobrindo o caminho.
        const repeticoes = Math.ceil(comprimento / larguraUnidade) + 2;
        setM((v) =>
          v.comprimento === comprimento && v.unidade === larguraUnidade
            ? v
            : { comprimento, unidade: larguraUnidade, repeticoes }
        );
      } catch {
        /* o navegador ainda não mediu o caminho */
      }
    };
    medir();
    document.fonts?.ready.then(medir).catch(() => {});
  }, [d, unidade, tamanho]);

  useEffect(() => {
    const alvo = correnteRef.current;
    if (!alvo || !m.unidade) return;

    const aplicar = (o: number) => alvo.setAttribute("startOffset", String(o));
    aplicar(m.unidade);

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const estado = { o: m.unidade };
    const t = gsap.to(estado, {
      o: 0,
      duration: m.unidade / velocidade,
      ease: "none",
      repeat: -1,
      onUpdate: () => aplicar(estado.o),
    });
    return () => void t.kill();
  }, [m, velocidade]);

  const estilo = { fontSize: `${tamanho}px`, fontWeight: 560, letterSpacing: "1px" };

  return (
    <div className="texto-loop" aria-hidden>
      <svg viewBox={`0 0 ${L} ${A}`} preserveAspectRatio="xMidYMid meet">
        <path ref={pathRef} id={id} d={d} fill="none" />

        <text ref={medidaRef} className="texto-loop__medida" style={estilo}>
          {unidade}
        </text>

        <text className="texto-loop__texto" style={estilo} dominantBaseline="central">
          <textPath ref={correnteRef} href={`#${id}`} startOffset={0}>
            {unidade.repeat(m.repeticoes)}
          </textPath>
        </text>
      </svg>
    </div>
  );
}
