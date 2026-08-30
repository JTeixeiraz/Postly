// Traz o CSS do aplicativo para o site, escopado sob `.vitrine`.
//
// A vitrine mostra as telas de verdade, não capturas: mesmo markup, mesmo CSS,
// componentes vivos. Para isso o CSS do app precisa valer aqui dentro — mas só
// aqui dentro, senão `.btn` e `.topo` do app sobrescrevem os do site.
//
// É um passo de build, não uma cópia manual: cópia manual envelhece no dia em
// que alguém mexe no app e ninguém lembra de refazer.
import { readFileSync, writeFileSync } from "node:fs";

// A classe repetida dobra a especificidade de propósito. Sem isto, `.topo` e
// `.btn` do site — que têm um nome só, como os do app — empatam com
// `.vitrine .topo` e vencem por vir depois no bundle, e o cabeçalho do
// aplicativo aparece com o fundo do site. Renomear as classes do site
// resolveria as duas de hoje e voltaria a quebrar na próxima que colidir.
const ESCOPO = ".vitrine.vitrine";
const FONTES = ["styles.css", "components.css", "config.css"];
const FONTES_SITE = ["estilo.css", "componentes.css"];
const SUFIXO_ANIM = "-vit"; // keyframes são globais: renomear evita colidir

function escopar(css) {
  // Fora: os @import de fonte (o site já carrega Geist) e os comentários, que
  // atrapalham a varredura de chaves abaixo.
  css = css.replace(/@import[^;]+;/g, "").replace(/\/\*[\s\S]*?\*\//g, "");

  const nomesAnim = new Set(
    [...css.matchAll(/@keyframes\s+([\w-]+)/g)].map((m) => m[1])
  );

  const saida = [];
  let i = 0;
  while (i < css.length) {
    const abre = css.indexOf("{", i);
    if (abre === -1) break;

    const seletor = css.slice(i, abre).trim();
    // Bloco com aninhamento (@media, @supports, @keyframes): recorta o corpo
    // inteiro contando chaves e trata cada caso.
    if (seletor.startsWith("@")) {
      let nivel = 0, j = abre;
      do { if (css[j] === "{") nivel++; else if (css[j] === "}") nivel--; j++; } while (nivel > 0 && j < css.length);
      const corpo = css.slice(abre + 1, j - 1);
      if (/^@(media|supports|layer|container)/.test(seletor)) {
        saida.push(`${seletor}{\n${escopar(corpo)}\n}`);
      } else if (seletor.startsWith("@keyframes")) {
        saida.push(seletor.replace(/(@keyframes\s+)([\w-]+)/, `$1$2${SUFIXO_ANIM}`) + `{${corpo}}`);
      } else {
        saida.push(`${seletor}{${corpo}}`);
      }
      i = j;
      continue;
    }

    const fim = css.indexOf("}", abre);
    let corpo = css.slice(abre + 1, fim);
    // As animações renomeadas precisam ser renomeadas também em quem as usa.
    for (const n of nomesAnim) {
      corpo = corpo.replace(
        new RegExp(`(animation(?:-name)?\\s*:[^;]*?)\\b${n}\\b`, "g"),
        `$1${n}${SUFIXO_ANIM}`
      );
    }

    const alvos = seletor
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean)
      .map((s) => {
        // Os tokens do app moram em :root; aqui eles passam a valer na vitrine.
        if (s === ":root" || s === "html" || s === "body") return ESCOPO;
        return `${ESCOPO} ${s}`;
      });

    saida.push(`${alvos.join(", ")}{${corpo}}`);
    i = fim + 1;
  }
  return saida.join("\n");
}

const partes = FONTES.map((f) => {
  const css = readFileSync(new URL(`../../src/${f}`, import.meta.url), "utf8");
  return `/* ── de src/${f} ─────────────────────────────────── */\n${escopar(css)}`;
});

writeFileSync(
  new URL("../src/vitrine-app.css", import.meta.url),
  `/* GERADO por scripts/importar-css.mjs — não edite à mão.\n` +
    `   Fonte: ../src/*.css do aplicativo, escopado sob ${ESCOPO}. */\n\n` +
    partes.join("\n\n") +
    "\n"
);
// Guarda contra a regressão que já aconteceu uma vez: `.posta` do site tinha
// `width: 14px` e o app não declarava largura nenhuma, então a estação do
// revezamento virava um ponto de 14px dentro da vitrine. Especificidade não
// resolve esse caso — quando um lado declara a propriedade e o outro não, ela
// aplica de qualquer jeito. A saída é renomear a classe do site.
const nomes = (css, prefixo) => {
  const mapa = new Map();
  for (const m of css.replace(/\/\*[\s\S]*?\*\//g, "").matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const [, sel, corpo] = m;
    if (sel.trim().startsWith("@") || !corpo.trim()) continue;
    const props = new Set(corpo.split(";").filter((x) => x.includes(":")).map((x) => x.split(":")[0].trim()));
    for (let s of sel.split(",")) {
      s = s.trim();
      if (prefixo && s.startsWith(prefixo)) s = s.slice(prefixo.length).trim();
      const cls = [...s.matchAll(/\.([a-zA-Z][\w-]*)/g)].map((x) => x[1]);
      if (cls.length) {
        const k = cls[cls.length - 1];
        mapa.set(k, new Set([...(mapa.get(k) ?? []), ...props]));
      }
    }
  }
  return mapa;
};

const doSite = nomes(
  FONTES_SITE.map((f) => readFileSync(new URL(`../src/${f}`, import.meta.url), "utf8")).join("\n"),
  null
);
const daVitrine = nomes(readFileSync(new URL("../src/vitrine-app.css", import.meta.url), "utf8"), ESCOPO);

const RISCO = /^(width|height|position|display|background|color|padding|margin|border|font|flex|grid|transform|inset|top|left|right|bottom|overflow)/;
const conflitos = [];
for (const [cls, props] of doSite) {
  const noApp = daVitrine.get(cls);
  if (!noApp) continue;
  const vaza = [...props].filter((p) => !noApp.has(p) && RISCO.test(p));
  if (vaza.length) conflitos.push(`  .${cls} — o site declara ${vaza.join(", ")} e o app não`);
}
if (conflitos.length) {
  console.error("\nCOLISÃO: o CSS do site vaza para dentro da vitrine.");
  console.error("Renomeie a classe no site (ver `.espinha__posta`, `.cabecalho`, `.acao`).\n");
  console.error(conflitos.join("\n"));
  process.exit(1);
}

console.log("vitrine-app.css gerado — sem colisão com o CSS do site");
