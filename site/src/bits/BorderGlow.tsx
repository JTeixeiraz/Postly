import { useRef, useCallback, type ReactNode } from "react";
import "./BorderGlow.css";

/** Cartão com brilho de borda que segue o ponteiro.
 *
 *  De reactbits.dev, reduzido ao que este projeto usa: uma cor só (a de ação),
 *  sem o gradiente de malha de sete pontos do original — sete matizes num
 *  sistema que tem um acento seria outro sistema. */
export default function BorderGlow({
  children,
  className = "",
  sensibilidade = 26,
  raio = 18,
  alcance = 44,
}: {
  children: ReactNode;
  className?: string;
  sensibilidade?: number;
  raio?: number;
  alcance?: number;
}) {
  const ref = useRef<HTMLDivElement>(null);

  const mover = useCallback((e: React.PointerEvent) => {
    const card = ref.current;
    if (!card) return;
    const r = card.getBoundingClientRect();
    const x = e.clientX - r.left;
    const y = e.clientY - r.top;
    const cx = r.width / 2;
    const cy = r.height / 2;
    const dx = x - cx;
    const dy = y - cy;

    // Proximidade da borda: 0 no centro, 1 encostado. É a razão entre a
    // distância percorrida e a distância até a borda naquela direção.
    const kx = dx !== 0 ? cx / Math.abs(dx) : Infinity;
    const ky = dy !== 0 ? cy / Math.abs(dy) : Infinity;
    const borda = Math.min(Math.max(1 / Math.min(kx, ky), 0), 1);

    let ang = Math.atan2(dy, dx) * (180 / Math.PI) + 90;
    if (ang < 0) ang += 360;

    card.style.setProperty("--bg-borda", (borda * 100).toFixed(2));
    card.style.setProperty("--bg-angulo", `${ang.toFixed(2)}deg`);
  }, []);

  return (
    <div
      ref={ref}
      onPointerMove={mover}
      className={`brilho-borda ${className}`.trim()}
      style={
        {
          "--bg-sensibilidade": sensibilidade,
          "--bg-raio": `${raio}px`,
          "--bg-alcance": `${alcance}px`,
        } as React.CSSProperties
      }
    >
      <span className="brilho-borda__luz" aria-hidden />
      <div className="brilho-borda__dentro">{children}</div>
    </div>
  );
}
