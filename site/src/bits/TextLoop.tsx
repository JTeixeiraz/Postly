import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { gsap } from "gsap";
import "./TextLoop.css";

const L = 1200;
// Altura justa para a onda: um viewBox alto deixa o texto num terço central
// com vazio em cima e embaixo, e a faixa vira um vão na página.
const A = 110;
const CY = A / 2;

/** Texto correndo por uma curva.
 *
 *  De reactbits.dev, com a onda achatada e a faixa removida: o original desenha
 *  uma fita colorida atrás das letras, e aqui a faixa competiria com o acento
 *  que já marca a ação. Fica só o texto, ondulando entre as seções.
 *
 *  Dois `<textPath>` alternados dão a volta sem emenda: quando o primeiro sai
 *  pela direita, o segundo já entrou pela esquerda. */
export default function TextLoop({
  texto = "código aberto",
  separador = "✦",
  velocidade = 46,
  amplitude = 26,
  tamanho = 30,
}: {
  texto?: string;
  separador?: string;
  velocidade?: number;
  amplitude?: number;
  tamanho?: number;
}) {
  const raizRef = useRef<HTMLDivElement>(null);
  const pathRef = useRef<SVGPathElement>(null);
  const medidaRef = useRef<SVGTextElement>(null);
  const aRef = useRef<SVGTextPathElement>(null);
  const bRef = useRef<SVGTextPathElement>(null);
  const [m, setM] = useState({ comprimento: 0, repeticoes: 1 });

  const id = `loop-${useId().replace(/:/g, "")}`;
  const d = useMemo(
    () =>
      `M -320 ${CY} Q -160 ${CY - amplitude} 0 ${CY} T 320 ${CY} T 640 ${CY} T 960 ${CY} T ${L + 320} ${CY}`,
    [amplitude]
  );
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
        const largura = med.getComputedTextLength();
        if (!comprimento || !largura) return;
        setM({ comprimento, repeticoes: Math.max(1, Math.round(comprimento / largura)) });
      } catch {
        /* o path ainda não foi medido pelo navegador */
      }
    };
    medir();
    document.fonts?.ready.then(medir).catch(() => {});
  }, [d, unidade, tamanho]);

  useEffect(() => {
    const { comprimento } = m;
    const a = aRef.current;
    const b = bRef.current;
    if (!a || !b || !comprimento) return;

    const aplicar = (o: number) => {
      a.setAttribute("startOffset", String(o));
      b.setAttribute("startOffset", String(o >= 0 ? o - comprimento : o + comprimento));
    };
    aplicar(0);

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const estado = { o: 0 };
    const t = gsap.to(estado, {
      o: comprimento,
      duration: comprimento / velocidade,
      ease: "none",
      repeat: -1,
      onUpdate: () => aplicar(estado.o),
    });
    return () => void t.kill();
  }, [m, velocidade]);

  const corrente = unidade.repeat(m.repeticoes);
  const estilo = { fontSize: `${tamanho}px`, fontWeight: 560, letterSpacing: "1px" };

  return (
    <div ref={raizRef} className="texto-loop" aria-hidden>
      <svg viewBox={`0 0 ${L} ${A}`} preserveAspectRatio="xMidYMid meet">
        <path ref={pathRef} id={id} d={d} fill="none" />
        <text ref={medidaRef} className="texto-loop__medida" style={estilo}>
          {unidade}
        </text>
        {[aRef, bRef].map((r, i) => (
          <text key={i} className="texto-loop__texto" style={estilo} dominantBaseline="central">
            <textPath
              ref={r}
              href={`#${id}`}
              startOffset={0}
              textLength={m.comprimento || undefined}
              lengthAdjust="spacing"
            >
              {corrente}
            </textPath>
          </text>
        ))}
      </svg>
    </div>
  );
}
