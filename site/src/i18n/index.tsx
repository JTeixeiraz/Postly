// Idioma da página: detectado no navegador, trocável no cabeçalho.
//
// O mesmo desenho do aplicativo, sem a parte que fala com o Rust. Quem chega
// de um link em inglês lê em inglês sem clicar em nada; quem prefere o outro
// idioma troca uma vez e a escolha fica.

import {
  createContext, useCallback, useContext, useEffect, useMemo, useState,
  type ReactNode,
} from "react";
import { pt, type Dicionario } from "./pt";
import { en } from "./en";

export type Idioma = "pt" | "en";

const DICIONARIOS: Record<Idioma, Dicionario> = { pt, en };
const CHAVE = "postly-site:idioma";

/** A escolha guardada vence a do navegador: ela é mais recente e mais
 *  específica. Sem armazenamento — navegador com cookies bloqueados, aba
 *  anônima — a detecção continua funcionando, que é o caso comum. */
function detectar(): Idioma {
  try {
    const salvo = localStorage.getItem(CHAVE);
    if (salvo === "pt" || salvo === "en") return salvo;
  } catch {
    // Armazenamento bloqueado: cai na detecção.
  }
  // `languages` respeita a ordem de preferência configurada; `language` é o
  // reserva para navegadores que não expõem a lista.
  const preferidos = navigator.languages?.length
    ? navigator.languages
    : [navigator.language ?? ""];
  return preferidos.some((l) => l.toLowerCase().startsWith("pt")) ? "pt" : "en";
}

interface Contexto {
  idioma: Idioma;
  trocar: (i: Idioma) => void;
  d: Dicionario;
}

const Ctx = createContext<Contexto | null>(null);

export function ProvedorIdioma({ children }: { children: ReactNode }) {
  const [idioma, setIdioma] = useState<Idioma>(detectar);

  const trocar = useCallback((novo: Idioma) => {
    setIdioma(novo);
    try {
      localStorage.setItem(CHAVE, novo);
    } catch {
      // Preferência não persistida é um incômodo, não um erro.
    }
  }, []);

  // O `lang` do documento e os metadados acompanham a troca. Não é detalhe:
  // é o que diz ao leitor de tela em que idioma pronunciar a página, e o que
  // um buscador lê ao indexar.
  useEffect(() => {
    const d = DICIONARIOS[idioma];
    document.documentElement.lang = d.meta.lang;
    document.title = d.meta.titulo;
    const meta = (sel: string, valor: string) =>
      document.querySelector(sel)?.setAttribute("content", valor);
    meta('meta[name="description"]', d.meta.descricao);
    meta('meta[property="og:description"]', d.meta.ogDescricao);
  }, [idioma]);

  const valor = useMemo<Contexto>(
    () => ({ idioma, trocar, d: DICIONARIOS[idioma] }),
    [idioma, trocar]
  );

  return <Ctx.Provider value={valor}>{children}</Ctx.Provider>;
}

export function useIdioma(): Contexto {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useIdioma fora do ProvedorIdioma");
  return ctx;
}
