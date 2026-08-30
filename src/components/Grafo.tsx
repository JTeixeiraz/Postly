import { useEffect, useRef } from "react";
import type { GrafoCerebro } from "../types";

interface Props {
  grafo: GrafoCerebro;
  selecionado: string | null;
  onSelecionar: (id: string) => void;
}

interface Ponto {
  id: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  grau: number;
  /** Posicionado a mao: a fisica para de mandar neste node. */
  fixo?: boolean;
}

// Espelham os tokens do CSS: o canvas nao le variavel, entao os valores vivem
// aqui e mudam junto quando a paleta muda.
// Espelham os tokens do CSS: o canvas nao le variavel, entao os valores vivem
// aqui e mudam junto quando a paleta muda.
const CORES = {
  fundo: "oklch(0.288 0.009 250)",
  no: "oklch(0.760 0.006 250)",
  noAceso: "oklch(0.880 0.200 124)",
  texto: "oklch(0.760 0.006 250)",
  textoAceso: "oklch(0.975 0.002 250)",
};

/** O peso da aresta vira cor, não só espessura.
 *
 *  Espessura sozinha não se lê num fio de 1px: o grafo inteiro parecia
 *  uniforme. Aqui a relação forte puxa para o âmbar e a fraca fica no cinza da
 *  superfície, que é a mesma gramática do resto da interface. */
function corDoPeso(peso: number, aceso: boolean): string {
  // Em superficie escura a relacao forte fica mais CLARA e mais saturada: e o
  // oposto do que valia no papel branco, e sem inverter isso a aresta que mais
  // importa seria a que menos aparece.
  const l = 0.42 + peso * 0.44 + (aceso ? 0.05 : 0);
  const c = (0.004 + peso * 0.13) * (aceso ? 1.3 : 1);
  return `oklch(${l.toFixed(3)} ${c.toFixed(3)} 124)`;
}

/** Grafo dirigido por forças, em canvas.
 *
 *  Sem biblioteca: são poucas centenas de nodes, e duzentas linhas de canvas
 *  pesam menos que qualquer dependência de visualização. O layout redimensiona
 *  junto com o contêiner, então a janela pode mudar de forma sem quebrar o
 *  desenho. */
export default function Grafo({ grafo, selecionado, onSelecionar }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const pontosRef = useRef<Ponto[]>([]);
  const hoverRef = useRef<string | null>(null);
  const selRef = useRef<string | null>(selecionado);
  selRef.current = selecionado;
  // Enquadramento: converte a posicao fisica de um node na posicao desenhada.
  // O hit-test do mouse precisa desfazer a mesma conta, por isso ele vive num
  // ref em vez de ficar preso ao laco de animacao.
  const vistaRef = useRef({ e: 1, cx: 0, cy: 0, ox: 0, oy: 0 });
  // Navegacao manual. Enquanto `livre` e falso o enquadramento automatico
  // manda; no primeiro arrasto ou zoom o controle passa para a pessoa, e o
  // duplo clique devolve. Sem isso, mexer no grafo seria uma briga contra o
  // laco de animacao, que recentraliza tudo a cada quadro.
  const navRef = useRef({ dx: 0, dy: 0, zoom: 1, livre: false });
  const arrastoRef = useRef<{ x: number; y: number } | null>(null);
  /** Distingue clique de arrasto: so o clique seco seleciona um node. */
  const arrastouRef = useRef(false);
  /** Node sendo arrastado agora, se houver. */
  const noArrastadoRef = useRef<string | null>(null);
  /** Acorda a fisica quando algo muda de lugar. */
  const acordarRef = useRef<() => void>(() => {});

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let largura = 0;
    let altura = 0;
    let anim = 0;
    let energia = 1;
    // Distancia caracteristica entre nodes, derivada da area disponivel. E o
    // que faz o mesmo grafo ocupar bem uma faixa larga e uma janela estreita,
    // em vez de se amontoar no centro deixando metade da tela vazia.
    let k = 90;

    const ids = Object.keys(grafo.nodes);
    const grau = new Map<string, number>();
    for (const a of grafo.edges) {
      grau.set(a.from, (grau.get(a.from) ?? 0) + 1);
      grau.set(a.to, (grau.get(a.to) ?? 0) + 1);
    }

    const semear = () => {
      // Elipse, nao circulo: a semente ja nasce com a proporcao do container,
      // entao o layout converge ocupando a forma que existe.
      pontosRef.current = ids.map((id, i) => {
        const ang = (i / Math.max(ids.length, 1)) * Math.PI * 2;
        return {
          id,
          x: largura / 2 + Math.cos(ang) * (largura / 2.6),
          y: altura / 2 + Math.sin(ang) * (altura / 2.6),
          vx: 0,
          vy: 0,
          grau: grau.get(id) ?? 0,
        };
      });
      energia = 1;
    };

    const medir = () => {
      const dpr = window.devicePixelRatio || 1;
      const novaL = canvas.clientWidth;
      const novaA = canvas.clientHeight;
      if (novaL === largura && novaA === altura) return;
      const primeira = largura === 0;
      largura = novaL;
      altura = novaA;
      k = Math.max(46, Math.sqrt((largura * altura) / Math.max(ids.length, 1)) * 0.62);
      canvas.width = Math.round(largura * dpr);
      canvas.height = Math.round(altura * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      if (primeira) semear();
      else energia = Math.max(energia, 0.55); // reacomoda depois do resize
    };

    const observer = new ResizeObserver(medir);
    observer.observe(canvas);
    medir();

    const porId = () => new Map(pontosRef.current.map((p) => [p.id, p]));

    const passo = () => {
      const pontos = pontosRef.current;
      const mapa = porId();

      if (energia > 0.002) {
        for (let i = 0; i < pontos.length; i++) {
          for (let j = i + 1; j < pontos.length; j++) {
            const a = pontos[i];
            const b = pontos[j];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let d2 = dx * dx + dy * dy;
            if (d2 < 1) {
              d2 = 1;
              dx = Math.random() - 0.5;
              dy = Math.random() - 0.5;
            }
            const d = Math.sqrt(d2);
            // Repulsao em k²/d: a escala acompanha o tamanho do container, e o
            // grafo se espalha pela area que existe em vez de por um numero
            // fixo calibrado numa janela so.
            const forca = (k * k) / d;
            a.vx -= (dx / d) * forca * 0.012;
            a.vy -= (dy / d) * forca * 0.012;
            b.vx += (dx / d) * forca * 0.012;
            b.vy += (dy / d) * forca * 0.012;
          }
        }

        // Atração proporcional ao peso: relação forte encurta a distância.
        for (const aresta of grafo.edges) {
          const a = mapa.get(aresta.from);
          const b = mapa.get(aresta.to);
          if (!a || !b) continue;
          const dx = b.x - a.x;
          const dy = b.y - a.y;
          const d = Math.max(Math.hypot(dx, dy), 1);
          const alvo = k * (1.45 - aresta.weight * 0.65);
          const forca = (d - alvo) * 0.018;
          a.vx += (dx / d) * forca;
          a.vy += (dy / d) * forca;
          b.vx -= (dx / d) * forca;
          b.vy -= (dy / d) * forca;
        }

        // Sem paredes. O grafo nao cabe mais dentro do retangulo do canvas:
        // ele se estende ate onde precisar, e o enquadramento automatico
        // reajusta a escala para tudo continuar visivel. Quem quiser olhar de
        // perto usa o zoom e o arrasto. Prender node em borda distorcia o
        // layout: dois nodes empurrados contra a mesma parede ficavam colados
        // fingindo uma proximidade que a relacao entre eles nao tinha.
        for (const p of pontos) {
          // Node posto a mao fica onde foi solto: ele continua puxando os
          // vizinhos pelas molas, mas nao e mais empurrado por elas. Sem isto,
          // arrastar seria uma sugestao que a fisica desfaz em dois quadros.
          if (p.fixo) {
            p.vx = 0;
            p.vy = 0;
            continue;
          }
          // A gravidade central substitui as paredes: ela e fraca o bastante
          // para nao achatar o layout e forte o bastante para nada escapar
          // para o infinito.
          p.vx += (largura / 2 - p.x) * 0.0022;
          p.vy += (altura / 2 - p.y) * 0.0022;
          p.vx *= 0.85;
          p.vy *= 0.85;
          // Teto de velocidade. Arrastar um node cria uma distancia subita, a
          // mola responde com forca proporcional a ela, e sem este limite os
          // vizinhos sao arremessados para fora da area num unico quadro.
          const TETO = 26;
          const vel = Math.hypot(p.vx, p.vy);
          if (vel > TETO) {
            p.vx = (p.vx / vel) * TETO;
            p.vy = (p.vy / vel) * TETO;
          }
          p.x += p.vx * energia;
          p.y += p.vy * energia;
        }
        energia *= 0.988;
      }

      // ---- enquadramento ----
      // O layout por forcas resolve as posicoes relativas; ele nao sabe nada
      // sobre a forma do container. Sem este passo o mesmo grafo deixa metade
      // de uma janela larga vazia. As margens sao assimetricas porque o rotulo
      // do node cresce para a direita.
      if (pontos.length > 0) {
        let minX = Infinity;
        let maxX = -Infinity;
        let minY = Infinity;
        let maxY = -Infinity;
        for (const p of pontos) {
          if (p.x < minX) minX = p.x;
          if (p.x > maxX) maxX = p.x;
          if (p.y < minY) minY = p.y;
          if (p.y > maxY) maxY = p.y;
        }
        const mE = 30;
        const mD = 104;
        const mV = 28;
        const alvoE = Math.min(
          (largura - mE - mD) / Math.max(maxX - minX, 1),
          (altura - mV * 2) / Math.max(maxY - minY, 1),
          2.2
        );
        const v = vistaRef.current;
        // Enquanto a pessoa conduz, o enquadramento congela: sem isto o laco
        // recentralizaria o grafo a cada quadro e o arrasto seria uma briga.
        if (!navRef.current.livre) {
          // Suavizado: acompanha o layout se acomodando em vez de saltar a
          // cada quadro enquanto os nodes ainda se movem.
          v.e += (alvoE - v.e) * 0.07;
          v.cx += ((minX + maxX) / 2 - v.cx) * 0.07;
          v.cy += ((minY + maxY) / 2 - v.cy) * 0.07;
        }
        // O centro da tela depende do tamanho do canvas, nao da navegacao:
        // continua valendo mesmo com a pessoa conduzindo.
        v.ox = (largura + mE - mD) / 2;
        v.oy = altura / 2;
      }
      const { e: esc, cx, cy, ox, oy } = vistaRef.current;
      const nav = navRef.current;
      const px = (p: Ponto) => (p.x - cx) * esc * nav.zoom + ox + nav.dx;
      const py = (p: Ponto) => (p.y - cy) * esc * nav.zoom + oy + nav.dy;

      // ---- desenho ----
      ctx.clearRect(0, 0, largura, altura);
      const foco = hoverRef.current ?? selRef.current;

      ctx.lineCap = "round";
      for (const aresta of grafo.edges) {
        const a = mapa.get(aresta.from);
        const b = mapa.get(aresta.to);
        if (!a || !b) continue;
        const ligado = foco === aresta.from || foco === aresta.to;
        ctx.strokeStyle = corDoPeso(aresta.weight, ligado);
        ctx.globalAlpha = foco && !ligado ? 0.3 : 1;
        ctx.lineWidth = (ligado ? 1.4 : 0.9) + aresta.weight * 2.1;
        const ax = px(a);
        const ay = py(a);
        const bx = px(b);
        const by = py(b);
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(bx, by);
        ctx.stroke();

        // O numero do peso so aparece nas arestas do node em foco: e a resposta
        // exata que a vizinhanca ordenada devolve ao agente.
        if (ligado) {
          ctx.font = `10px "Geist Mono Variable", ui-monospace, monospace`;
          ctx.textAlign = "center";
          ctx.lineWidth = 3.5;
          ctx.lineJoin = "round";
          ctx.strokeStyle = CORES.fundo;
          ctx.strokeText(aresta.weight.toFixed(2), (ax + bx) / 2, (ay + by) / 2 - 4);
          ctx.fillStyle = CORES.texto;
          ctx.fillText(aresta.weight.toFixed(2), (ax + bx) / 2, (ay + by) / 2 - 4);
          ctx.textAlign = "left";
        }
      }
      ctx.globalAlpha = 1;

      for (const p of pontos) {
        const ativo = p.id === foco;
        const raio = 3.8 + Math.min(p.grau, 9) * 0.78;
        const x = px(p);
        const y = py(p);

        // Halo: separa o node do fio que passa atras dele e marca o foco sem
        // precisar de uma segunda cor.
        if (ativo) {
          ctx.fillStyle = "oklch(0.880 0.200 124 / 0.20)";
          ctx.beginPath();
          ctx.arc(x, y, raio + 7, 0, Math.PI * 2);
          ctx.fill();
        }

        ctx.fillStyle = ativo ? CORES.noAceso : CORES.no;
        ctx.beginPath();
        ctx.arc(x, y, raio, 0, Math.PI * 2);
        ctx.fill();

        // Node preso ganha um anel: sem marca, a pessoa nao sabe quais ficaram
        // onde ela colocou e quais o layout ainda pode mover.
        if (p.fixo) {
          ctx.strokeStyle = CORES.noAceso;
          ctx.lineWidth = 1.6;
          ctx.beginPath();
          ctx.arc(x, y, raio + 4, 0, Math.PI * 2);
          ctx.stroke();
        }

        if (ativo || p.grau >= 2 || pontos.length <= 20) {
          ctx.font = `${ativo ? 11.5 : 10.5}px "Geist Mono Variable", ui-monospace, monospace`;
          ctx.globalAlpha = ativo ? 1 : 0.8;
          // Contorno na cor da superficie: sem isto o fio passa por dentro da
          // palavra e o nome do node fica ilegivel justamente no aglomerado,
          // que e onde ele mais importa.
          ctx.lineWidth = 3.5;
          ctx.strokeStyle = CORES.fundo;
          ctx.lineJoin = "round";
          ctx.strokeText(p.id, x + raio + 6, y + 3.5);
          ctx.fillStyle = ativo ? CORES.textoAceso : CORES.texto;
          ctx.fillText(p.id, x + raio + 6, y + 3.5);
          ctx.globalAlpha = 1;
        }
      }

      anim = requestAnimationFrame(passo);
    };

    // Arrastar um node reacomoda os vizinhos: sem reacender a energia, o
    // grafo ja teria esfriado e nada se moveria em volta.
    acordarRef.current = () => {
      // Baixo de proposito: o suficiente para os vizinhos se reacomodarem,
      // nao para o grafo inteiro reorganizar a cada pixel arrastado.
      energia = Math.max(energia, 0.22);
    };

    anim = requestAnimationFrame(passo);
    return () => {
      cancelAnimationFrame(anim);
      observer.disconnect();
    };
  }, [grafo]);

  /** Da tela para o espaco da fisica: o inverso exato da projecao do desenho. */
  const paraFisica = (x: number, y: number) => {
    const v = vistaRef.current;
    const n = navRef.current;
    const e = v.e * n.zoom || 1;
    return { x: (x - v.ox - n.dx) / e + v.cx, y: (y - v.oy - n.dy) / e + v.cy };
  };

  const maisProximo = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - r.left;
    const y = e.clientY - r.top;
    let achado: { id: string; d: number } | null = null;
    const v = vistaRef.current;
    const n = navRef.current;
    for (const p of pontosRef.current) {
      const d = Math.hypot(
        (p.x - v.cx) * v.e * n.zoom + v.ox + n.dx - x,
        (p.y - v.cy) * v.e * n.zoom + v.oy + n.dy - y
      );
      if (d < 16 && (!achado || d < achado.d)) achado = { id: p.id, d };
    }
    return achado?.id ?? null;
  };

  return (
    <canvas
      ref={canvasRef}
      className="grafo"
      onClick={(e) => {
        // Clique que veio de um arrasto nao seleciona: sem esta guarda,
        // mover a vista trocaria o node escolhido sem querer.
        if (arrastouRef.current) return;
        const id = maisProximo(e);
        if (id) onSelecionar(id);
      }}
      onMouseDown={(e) => {
        arrastoRef.current = { x: e.clientX, y: e.clientY };
        arrastouRef.current = false;
        // Pegar um node move o node; pegar o vazio move a vista.
        noArrastadoRef.current = maisProximo(e);
      }}
      onMouseMove={(e) => {
        const inicio = arrastoRef.current;
        if (inicio) {
          const dx = e.clientX - inicio.x;
          const dy = e.clientY - inicio.y;
          if (Math.hypot(dx, dy) > 3) {
            arrastouRef.current = true;
            const alvo = noArrastadoRef.current;
            if (alvo) {
              // Arrastar node: leva o ponto para onde o cursor esta e o
              // prende ali. Os vizinhos reagem pelas molas.
              const cv = e.currentTarget.getBoundingClientRect();
              const destino = paraFisica(e.clientX - cv.left, e.clientY - cv.top);
              const p = pontosRef.current.find((x) => x.id === alvo);
              if (p) {
                // Sem congelar o enquadramento, cada quadro reposiciona a vista,
                // a conversao tela->fisica muda junto e o node persegue um alvo
                // que se move: o resultado e o grafo escapando da area.
                // O enquadramento continua vivo de proposito. Congelar durante
                // o arrasto prendia a escala no zoom anterior, e os vizinhos
                // empurrados pelas molas saiam da area sem a vista reagir. O
                // atraso na conversao tela->fisica e imperceptivel porque o
                // enquadramento anda so 7% por quadro.
                p.x = destino.x;
                p.y = destino.y;
                p.vx = 0;
                p.vy = 0;
                p.fixo = true;
                acordarRef.current();
              }
            } else {
              navRef.current.livre = true;
              navRef.current.dx += dx;
              navRef.current.dy += dy;
            }
            arrastoRef.current = { x: e.clientX, y: e.clientY };
          }
          e.currentTarget.style.cursor = "grabbing";
          return;
        }
        hoverRef.current = maisProximo(e);
        e.currentTarget.style.cursor = hoverRef.current ? "grab" : "default";
      }}
      onMouseUp={() => {
        arrastoRef.current = null;
        noArrastadoRef.current = null;
      }}
      onMouseLeave={() => {
        arrastoRef.current = null;
        noArrastadoRef.current = null;
        hoverRef.current = null;
      }}
      onWheel={(e) => {
        // Zoom ancorado no ponteiro: o que estava sob o cursor continua sob
        // ele. Zoom preso ao centro faz a pessoa perder o que estava olhando.
        const r = e.currentTarget.getBoundingClientRect();
        const mx = e.clientX - r.left;
        const my = e.clientY - r.top;
        const n = navRef.current;
        const fator = e.deltaY < 0 ? 1.12 : 1 / 1.12;
        const novo = Math.min(Math.max(n.zoom * fator, 0.35), 5);
        const real = novo / n.zoom;
        n.dx = mx - (mx - n.dx) * real;
        n.dy = my - (my - n.dy) * real;
        n.zoom = novo;
        n.livre = true;
      }}
      onDoubleClick={(e) => {
        // Sobre um node preso, solta ele. No vazio, reenquadra a vista.
        const alvo = maisProximo(e);
        if (alvo) {
          const p = pontosRef.current.find((x) => x.id === alvo);
          if (p?.fixo) {
            p.fixo = false;
            acordarRef.current();
            return;
          }
        }
        navRef.current = { dx: 0, dy: 0, zoom: 1, livre: false };
        for (const p of pontosRef.current) p.fixo = false;
        acordarRef.current();
      }}
    />
  );
}
