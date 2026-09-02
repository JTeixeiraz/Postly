import { useEffect, type DependencyList } from "react";

/** Mantém um ouvinte de evento do Tauri vivo pelo tempo de vida do componente.
 *
 * POR QUE ISTO NÃO É ESCRITO À MÃO EM CADA TELA. A função de cancelar chega
 * dentro de uma promessa, e a limpeza do efeito pode rodar antes dela. A forma
 * ingênua —
 *
 * ```ts
 * let parar: (() => void) | undefined;
 * void ouvir(...).then((x) => (parar = x));
 * return () => parar?.();
 * ```
 *
 * — cancela `undefined` nesse caso e o ouvinte fica vivo para sempre. Cada
 * remontagem soma mais uma cópia, e todas recebem o mesmo evento.
 *
 * Medido no arrastar-e-soltar de clipes: um vídeo solto uma vez foi parar duas
 * vezes na pasta do projeto (`take-arrastado.mp4` e `take-arrastado-2.mp4`, no
 * mesmo segundo). Onde o ouvinte empilha evento em lista — o log de estágios —
 * o efeito seria cada linha repetida.
 */
export function useOuvinte(
  abrir: () => Promise<() => void>,
  deps: DependencyList,
) {
  useEffect(() => {
    let vivo = true;
    let parar: (() => void) | undefined;
    void abrir().then((x) => {
      // Chegou depois da limpeza: ninguém mais vai chamar, então cancela aqui.
      if (vivo) parar = x;
      else x();
    });
    return () => {
      vivo = false;
      parar?.();
    };
    // `abrir` de propósito fora das dependências: quem chama declara em `deps`
    // o que de fato muda o ouvinte, e a função nova de cada render não muda.
  }, deps);
}
