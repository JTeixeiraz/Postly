import { useEffect, useId, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";

export interface Opcao {
  valor: string;
  rotulo: string;
  /** Segunda linha, quando a escolha precisa de explicacao. */
  nota?: string;
}

interface Props {
  valor: string;
  opcoes: Opcao[];
  onEscolher: (valor: string) => void;
  rotulo?: string;
  /** Conteudo a esquerda do rotulo, tipo a marca de uma familia de modelo. */
  antes?: React.ReactNode;
}

/** Seletor proprio.
 *
 *  O `<select>` nativo desenha a lista com o widget do sistema: no WebKitGTK
 *  ela sai como uma caixa cinza sem raio, sem a cor de acao e sem transicao,
 *  destoando de tudo em volta. Aqui a lista e do app: mesmos cantos, mesma cor
 *  de selecao e uma abertura curta.
 *
 *  O teclado continua funcionando como se espera de um seletor: setas para
 *  andar, Enter para escolher, Escape para fechar, Home e End para as pontas. */
export default function Selecao({ valor, opcoes, onEscolher, rotulo, antes }: Props) {
  const [aberto, setAberto] = useState(false);
  const [foco, setFoco] = useState(() => Math.max(0, opcoes.findIndex((o) => o.valor === valor)));
  const caixa = useRef<HTMLDivElement>(null);
  const id = useId();
  const atual = opcoes.find((o) => o.valor === valor) ?? opcoes[0];

  // Fechar ao clicar fora e ao rolar: uma lista que fica flutuando sobre
  // conteudo que se moveu e pior que uma lista que sumiu.
  useEffect(() => {
    if (!aberto) return;
    const fora = (e: MouseEvent) => {
      if (!caixa.current?.contains(e.target as Node)) setAberto(false);
    };
    document.addEventListener("mousedown", fora);
    return () => document.removeEventListener("mousedown", fora);
  }, [aberto]);

  const escolher = (v: string) => {
    onEscolher(v);
    setAberto(false);
  };

  const teclado = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setAberto(false);
      return;
    }
    if (!aberto && (e.key === "Enter" || e.key === " " || e.key === "ArrowDown")) {
      e.preventDefault();
      setAberto(true);
      setFoco(Math.max(0, opcoes.findIndex((o) => o.valor === valor)));
      return;
    }
    if (!aberto) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFoco((i) => Math.min(i + 1, opcoes.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFoco((i) => Math.max(i - 1, 0));
    } else if (e.key === "Home") {
      e.preventDefault();
      setFoco(0);
    } else if (e.key === "End") {
      e.preventDefault();
      setFoco(opcoes.length - 1);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      escolher(opcoes[foco].valor);
    }
  };

  return (
    <div className="selecao" ref={caixa}>
      <button
        type="button"
        className="selecao__gatilho"
        aria-haspopup="listbox"
        aria-expanded={aberto}
        aria-labelledby={rotulo ? `${id}-rot` : undefined}
        onClick={() => setAberto((a) => !a)}
        onKeyDown={teclado}
      >
        {antes}
        <span className="selecao__valor">{atual?.rotulo}</span>
        <span className="selecao__seta" data-aberto={aberto} aria-hidden />
      </button>

      <AnimatePresence>
        {aberto && (
          <motion.ul
            className="selecao__lista"
            role="listbox"
            initial={{ opacity: 0, y: -6, scale: 0.985 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.99 }}
            transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
          >
            {opcoes.map((o, i) => (
              <li key={o.valor}>
                <button
                  type="button"
                  role="option"
                  aria-selected={o.valor === valor}
                  className="selecao__op"
                  data-ativo={o.valor === valor}
                  data-foco={i === foco}
                  onMouseEnter={() => setFoco(i)}
                  onClick={() => escolher(o.valor)}
                >
                  <span className="selecao__op-rot">{o.rotulo}</span>
                  {o.nota && <span className="selecao__op-nota">{o.nota}</span>}
                </button>
              </li>
            ))}
          </motion.ul>
        )}
      </AnimatePresence>
    </div>
  );
}
