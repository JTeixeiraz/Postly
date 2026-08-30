// Traz componentes do aplicativo para o site, sem cópia manual.
//
// A vitrine mostra o produto de verdade — o grafo que você arrasta é o mesmo
// código que roda no app, não um vídeo dele. Estes componentes não dependem de
// Tauri nem do dicionário, então atravessam inteiros; os que dependem têm
// versão própria em src/vitrine/.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";

const COMPONENTES = ["Icons", "MarcaModelo", "Grafo"];
const DESTINO = new URL("../src/app/", import.meta.url);
mkdirSync(DESTINO, { recursive: true });

const CABECA = (nome) =>
  `/* GERADO por scripts/importar-componentes.mjs — não edite à mão.\n` +
  `   Fonte: ../../../src/components/${nome}.tsx */\n\n`;

for (const nome of COMPONENTES) {
  let src = readFileSync(new URL(`../../src/components/${nome}.tsx`, import.meta.url), "utf8");
  // O único desvio: os tipos moram em src/types.ts do app, que arrasta o resto
  // do mundo junto. A vitrine declara os seus.
  src = src.replace(/from "\.\.\/types"/g, 'from "../vitrine/tipos"');
  writeFileSync(new URL(`${nome}.tsx`, DESTINO), CABECA(nome) + src);
}
console.log("componentes importados:", COMPONENTES.join(", "));
