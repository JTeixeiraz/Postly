// Leitura de desempenho das publicacoes proprias.
//
// Separado dos adaptadores porque e outro assunto: la se PUBLICA, aqui se LE.
// E porque o modo de falhar tambem e outro — publicar errado manda um post
// torto para o mundo, ler errado envenena a mediana que decide a proxima
// campanha. Por isso tudo aqui devolve zero ou lista vazia quando duvida.

import { firstVisible } from "./networks.mjs";

//
// Le os numeros que ficam visiveis na propria peca: curtida e comentario. O
// ALCANCE nao entra aqui de proposito. Ele mora no painel profissional de cada
// rede, atras de mais cliques e de uma conta comercial, e uma raspagem que
// tenta chegar la falha muito mais do que acerta. A tela do app oferece o campo
// para digitar alcance a mao, e o registro raspado se junta ao digitado sem
// sobrescrever o que a pessoa colocou.
//
// Como todo o resto deste arquivo: quando o layout muda, isto devolve lista
// vazia em vez de numero errado. Numero errado envenena a mediana e a mediana
// e o que decide a proxima campanha.

/** Converte "1.234", "1,2 mil", "3.4K", "2M" no inteiro correspondente. */
export function paraNumero(texto) {
  if (!texto) return 0;
  const limpo = String(texto).trim().toLowerCase().replace(/\s+/g, " ");
  const m = limpo.match(/([\d.,]+)\s*(mil|k|m|mi|b)?/);
  if (!m) return 0;
  // Milhar com ponto ("1.234") e decimal com virgula ("1,2") convivem em pt-BR.
  // A regra: o ultimo separador seguido de 1 ou 2 digitos e decimal; o resto
  // e separador de milhar e sai fora.
  let num = m[1];
  const decimal = num.match(/[.,](\d{1,2})$/);
  if (decimal && /[.,]/.test(num.slice(0, -decimal[0].length))) {
    num = num.slice(0, -decimal[0].length).replace(/[.,]/g, "") + "." + decimal[1];
  } else if (decimal && num.replace(/[^.,]/g, "").length === 1) {
    num = num.replace(",", ".");
  } else {
    num = num.replace(/[.,]/g, "");
  }
  const base = parseFloat(num);
  if (!isFinite(base)) return 0;
  const escala = { mil: 1e3, k: 1e3, m: 1e6, mi: 1e6, b: 1e9 }[m[2]] || 1;
  return Math.round(base * escala);
}

/** Extrai os links das publicacoes proprias a partir da pagina de perfil. */
export async function linksDePosts(page, padrao, limite) {
  const hrefs = await page.evaluate((p) => {
    const vistos = new Set();
    for (const a of document.querySelectorAll("a[href]")) {
      if (new RegExp(p).test(a.getAttribute("href") || "")) vistos.add(a.href);
    }
    return [...vistos];
  }, padrao);
  return hrefs.slice(0, limite);
}

/** Le curtidas e comentarios de uma peca aberta. */
export async function numerosDaPagina(page, seletoresCurtida, seletoresComentario) {
  const ler = async (sel) => {
    const loc = await firstVisible(page, sel, 3500);
    if (!loc) return 0;
    return paraNumero((await loc.innerText().catch(() => "")) || "");
  };
  return {
    curtidas: await ler(seletoresCurtida),
    comentarios: await ler(seletoresComentario),
  };
}

/** Data da publicacao, quando a pagina expoe um <time datetime>. */
export async function dataDoPost(page) {
  const t = await page.locator("time[datetime]").first().getAttribute("datetime").catch(() => null);
  return (t || "").slice(0, 10);
}

