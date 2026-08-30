// Idioma da interface e da entrega dos agentes.
//
// Detecta pelo navegador na primeira abertura, guarda a escolha, e a mesma
// escolha viaja para o Rust: o que os modelos escrevem sai no idioma que a
// pessoa esta lendo.

import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { en, pt } from "./dict";

export type Idioma = "pt" | "en";
type Dicionario = typeof pt;

const DICIONARIOS: Record<Idioma, Dicionario> = { pt, en };
const CHAVE = "postly:idioma";

function detectar(): Idioma {
  try {
    const salvo = localStorage.getItem(CHAVE);
    if (salvo === "pt" || salvo === "en") return salvo;
  } catch {
    // Navegador com armazenamento bloqueado: cai na deteccao.
  }
  return navigator.language?.toLowerCase().startsWith("pt") ? "pt" : "en";
}

interface Contexto {
  idioma: Idioma;
  trocar: (i: Idioma) => void;
  d: Dicionario;
  /** Substitui {chave} pelos valores informados. */
  f: (modelo: string, valores: Record<string, string | number>) => string;
}

const Ctx = createContext<Contexto | null>(null);

export function ProvedorIdioma({ children }: { children: ReactNode }) {
  const [idioma, setIdioma] = useState<Idioma>(detectar);

  const trocar = useCallback((novo: Idioma) => {
    setIdioma(novo);
    // O backend tem as proprias mensagens: erro de campanha, recusa do Gemini,
    // aviso de memoria. Sem este aviso, metade da tela trocava de idioma e a
    // outra metade, justamente a que aparece quando algo da errado, nao.
    void invoke("definir_idioma", { idioma: novo }).catch(() => {});
    try {
      localStorage.setItem(CHAVE, novo);
    } catch {
      // Preferencia nao persistida e um incomodo, nao um erro.
    }
    document.documentElement.lang = novo === "pt" ? "pt-BR" : "en";
  }, []);

  // Tambem na abertura: a escolha pode ter vindo do armazenamento local.
  useEffect(() => {
    void invoke("definir_idioma", { idioma }).catch(() => {});
  }, [idioma]);

  const valor = useMemo<Contexto>(
    () => ({
      idioma,
      trocar,
      d: DICIONARIOS[idioma],
      f: (modelo, valores) =>
        modelo.replace(/\{(\w+)\}/g, (_, chave) =>
          chave in valores ? String(valores[chave]) : `{${chave}}`
        ),
    }),
    [idioma, trocar]
  );

  return <Ctx.Provider value={valor}>{children}</Ctx.Provider>;
}

export function useIdioma(): Contexto {
  const ctx = useContext(Ctx);
  if (!ctx) throw new Error("useIdioma precisa estar dentro de ProvedorIdioma");
  return ctx;
}

/** Bytes em unidade legivel, com o separador decimal do idioma. */
export function formatarBytes(bytes: number, idioma: Idioma): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const unidades = ["B", "KB", "MB", "GB", "TB"];
  let valor = bytes;
  let i = 0;
  while (valor >= 1024 && i < unidades.length - 1) {
    valor /= 1024;
    i += 1;
  }
  const texto = i === 0 ? String(Math.round(valor)) : valor.toFixed(1);
  return `${idioma === "pt" ? texto.replace(".", ",") : texto} ${unidades[i]}`;
}

export function formatarNumero(valor: number, idioma: Idioma, casas = 1): string {
  const texto = valor.toFixed(casas);
  return idioma === "pt" ? texto.replace(".", ",") : texto;
}

/** Id de execucao (`2026-08-29-1412`) em data legivel.
 *
 *  O id continua sendo a chave em disco, mas ninguem procura uma campanha por
 *  ele: procura pelo dia em que rodou. Se o formato nao bater, devolve o id
 *  cru em vez de inventar uma data. */
export function formatarExecucao(id: string, idioma: Idioma): string | null {
  // O id nasce em `start_run` como `%Y%m%d-%H%M%S`. A expressao anterior
  // esperava um formato que o Rust nunca gerou, entao TODA execucao caia no
  // retorno nulo e a tela mostrava o id cru achando que era uma data legivel.
  const m = /^(\d{4})(\d{2})(\d{2})-(\d{2})(\d{2})(\d{2})?$/.exec(id);
  if (!m) return null;
  const [, ano, mes, dia, hora, minuto] = m;
  const data = new Date(+ano, +mes - 1, +dia, +hora, +minuto);
  if (Number.isNaN(data.getTime())) return null;
  return new Intl.DateTimeFormat(idioma === "pt" ? "pt-BR" : "en-US", {
    day: "numeric",
    month: "long",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(data);
}
