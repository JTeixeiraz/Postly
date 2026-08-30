import { useEffect, useRef } from "react";

/** Fundo de trama animada.
 *
 *  Faz o papel do <Dither /> do reactbits sem a stack de WebGL: aquele traz
 *  three, postprocessing e dois pacotes de react-three (~150 KB gzip) e exige
 *  React 19, enquanto este site roda no 18 para casar com o aplicativo. Um
 *  fundo não justifica nenhuma das duas coisas.
 *
 *  O desenho é o mesmo em espírito — ruído de valor em octaves, quantizado com
 *  a matriz de Bayer 8×8, que é o que dá o aspecto de trama impressa em vez de
 *  gradiente liso. Roda a 30 fps num canvas de meia resolução. */
export default function Trama({
  altura = 520,
  passo = 3,
  velocidade = 0.00013,
}: {
  altura?: number;
  passo?: number;
  velocidade?: number;
}) {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d", { alpha: false });
    if (!ctx) return;

    const parado = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    // Bayer 8×8: o limiar ordenado que transforma uma rampa contínua em pontos.
    const bayer = [
      0, 48, 12, 60, 3, 51, 15, 63, 32, 16, 44, 28, 35, 19, 47, 31,
      8, 56, 4, 52, 11, 59, 7, 55, 40, 24, 36, 20, 43, 27, 39, 23,
      2, 50, 14, 62, 1, 49, 13, 61, 34, 18, 46, 30, 33, 17, 45, 29,
      10, 58, 6, 54, 9, 57, 5, 53, 42, 26, 38, 22, 41, 25, 37, 21,
    ].map((v) => v / 64);

    // Ruído de valor: hash inteiro + interpolação suave. Barato e determinístico.
    const hash = (x: number, y: number) => {
      const n = Math.sin(x * 127.1 + y * 311.7) * 43758.5453;
      return n - Math.floor(n);
    };
    const suave = (t: number) => t * t * (3 - 2 * t);
    const ruido = (x: number, y: number) => {
      const xi = Math.floor(x), yi = Math.floor(y);
      const xf = suave(x - xi), yf = suave(y - yi);
      const a = hash(xi, yi), b = hash(xi + 1, yi);
      const c = hash(xi, yi + 1), dd = hash(xi + 1, yi + 1);
      return a + (b - a) * xf + (c - a) * yf + (a - b - c + dd) * xf * yf;
    };

    let l = 0, a = 0, raf = 0;
    const medir = () => {
      const r = canvas.getBoundingClientRect();
      l = Math.max(1, Math.floor(r.width / passo));
      a = Math.max(1, Math.floor(altura / passo));
      canvas.width = l;
      canvas.height = a;
    };
    medir();

    const imagem = ctx.createImageData(l, a);
    const pintar = (t: number) => {
      const dados = imagem.data;
      for (let y = 0; y < a; y++) {
        for (let x = 0; x < l; x++) {
          const fx = x / l, fy = y / a;
          // Duas octaves bastam: a terceira só adiciona grão que a trama come.
          let v = ruido(fx * 3 + t, fy * 3.6) * 0.62 + ruido(fx * 7 - t * 1.6, fy * 8) * 0.3;
          // Escurece para as bordas: o conteúdo vive no meio e precisa de calma.
          v *= 1 - Math.min(1, Math.hypot(fx - 0.5, fy - 0.5) * 1.5);
          const nivel = v - bayer[(y % 8) * 8 + (x % 8)] * 0.30 > 0.19 ? 1 : 0;
          const i = (y * l + x) * 4;
          // Carvão do site, com o acento aparecendo só nos pontos acesos.
          dados[i] = nivel ? 92 : 22;
          dados[i + 1] = nivel ? 118 : 26;
          dados[i + 2] = nivel ? 30 : 30;
          dados[i + 3] = 255;
        }
      }
      ctx.putImageData(imagem, 0, 0);
    };

    pintar(0);
    if (!parado) {
      let ultimo = 0;
      const laco = (agora: number) => {
        // 30 fps: o desenho é feito na CPU e a 60 ele apareceria no perfil de
        // quem abre a página num notebook modesto.
        if (agora - ultimo > 33) {
          pintar(agora * velocidade);
          ultimo = agora;
        }
        raf = requestAnimationFrame(laco);
      };
      raf = requestAnimationFrame(laco);
    }

    const ro = new ResizeObserver(() => { medir(); pintar(performance.now() * velocidade); });
    ro.observe(canvas);
    return () => { cancelAnimationFrame(raf); ro.disconnect(); };
  }, [altura, passo, velocidade]);

  return <canvas ref={ref} className="trama" style={{ height: altura }} aria-hidden />;
}
