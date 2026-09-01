// Render do Remotion, chamado pelo Rust.
//
// UM PROCESSO POR RENDER, e não um canal vivo como o do Playwright. Um render
// é uma operação longa e única que termina com um arquivo; manter Node e
// Chromium de pé depois disso seria segurar centenas de MB para nada.
//
// Fala JSON-lines pelo stdout: uma linha por evento de progresso, e a última
// linha é o resultado. O Rust distingue as duas pelo formato.
//
// NENHUM DOWNLOAD NOVO. O Remotion baixa o próprio Chromium por padrão. O
// Postly já provisiona o do Playwright na primeira abertura, então o Rust manda
// o caminho dele em `chromium` e o render reusa. Sem isso, uma máquina que já
// tem dois navegadores em disco baixaria um terceiro — contra o mandato de que
// nada que a pessoa não pediu é baixado.

import { existsSync, readdirSync, mkdirSync } from "node:fs";
import { dirname, join, basename } from "node:path";

/** Uma linha de JSON no stdout. */
const emitir = (obj) => process.stdout.write(`${JSON.stringify(obj)}\n`);
const progresso = (fase, percent, detalhe = "") =>
  emitir({ fase, percent, detalhe });

async function main() {
  const pedido = JSON.parse(await lerEntrada());

  // O import é dinâmico e fica DENTRO do try: `@remotion/renderer` é pesado, e
  // um sidecar que morre no import de topo não consegue explicar por quê — a
  // mensagem sairia como um stack trace do Node no stderr, que é exatamente o
  // que o lado Rust filtra fora.
  let bundle, renderMedia, selectComposition;
  try {
    ({ bundle } = await import("@remotion/bundler"));
    ({ renderMedia, selectComposition } = await import("@remotion/renderer"));
  } catch (e) {
    return emitir({
      ok: false,
      erro:
        "O render do Remotion não está instalado no sidecar. Rode `npm ci --prefix sidecar`. " +
        String(e?.message ?? e),
    });
  }

  const { projeto, roteiro, saida, largura, altura, chromium, raiz_motion } = pedido;

  const entrada = join(raiz_motion, "src", "index.ts");
  if (!existsSync(entrada)) {
    return emitir({ ok: false, erro: `biblioteca de cenas não encontrada em ${entrada}` });
  }

  // OS ASSETS SÃO CAMINHOS RELATIVOS, NÃO URLs `file://`. Isto foi medido: a
  // primeira versão montava `pathToFileURL(...)` e o render falhava com
  //
  //   Not allowed to load local resource: file:///…/imagens/a.png
  //
  // O Chromium do render recusa `file://` por política de segurança, e a falha
  // só aparecia depois do bundle inteiro — 18% de um render que nunca ia
  // terminar. O caminho certo é a pasta do projeto virar o `publicDir` do
  // bundle, e a composição resolver cada nome com `staticFile()`.
  const assets = {};
  for (const sub of ["imagens", "audio"]) {
    const dir = join(projeto, sub);
    if (!existsSync(dir)) continue;
    for (const nome of readdirSync(dir)) {
      // Barra normal mesmo no Windows: isto vira parte de uma URL, não de um
      // caminho de disco.
      assets[nome] = `${sub}/${nome}`;
    }
  }

  // A narração vai em lista ordenada por nome, e não no mapa de assets: a
  // composição precisa TOCÁ-LA em ordem, e um objeto não garante ordem de
  // forma que dê para confiar num arquivo final.
  const dirVo = join(projeto, "narracao");
  const narracao = existsSync(dirVo)
    ? readdirSync(dirVo)
        .sort()
        .map((n) => `narracao/${n}`)
    : [];

  progresso("empacotando", 0, "montando a biblioteca de cenas");

  const bundleUrl = await bundle({
    entryPoint: entrada,
    // A pasta do projeto É o public dir deste bundle. Sem isto o Remotion usaria
    // `motion/public`, que não tem os assets de ninguém.
    publicDir: projeto,
    // Symlink em vez de cópia: o bundle é descartável (uma pasta temporária por
    // render), e copiar uma pasta de fotos a cada render gastaria disco e
    // minutos por nada. No Windows a opção é ignorada e a cópia acontece de
    // qualquer jeito — é o preço lá, não um bug aqui.
    symlinkPublicDir: true,
    onProgress: (p) => progresso("empacotando", p, ""),
  });

  const inputProps = { roteiro, assets, narracao };

  const composicao = await selectComposition({
    serveUrl: bundleUrl,
    id: "VideoDoUsuario",
    inputProps,
  });

  mkdirSync(dirname(saida), { recursive: true });
  progresso("renderizando", 0, basename(saida));

  await renderMedia({
    composition: {
      ...composicao,
      // As dimensões vêm do roteiro e não da composição: a proporção é escolha
      // de quem pediu o vídeo, e a composição tem um padrão só.
      width: largura,
      height: altura,
    },
    serveUrl: bundleUrl,
    codec: "h264",
    outputLocation: saida,
    inputProps,
    // `null` deixa o Remotion resolver sozinho — e aí ele pode baixar. O Rust
    // só manda `null` quando o Chromium do Playwright não está em disco, e a
    // tela avisa antes.
    browserExecutable: chromium ?? null,
    onProgress: ({ progress }) => progresso("renderizando", progress, ""),
  });

  emitir({
    ok: true,
    arquivo: saida,
    // Medida do que o arquivo TEM, e não do que o roteiro pediu. A diferença
    // importa: a soma das cenas é a intenção, isto é o resultado.
    duracao_s: composicao.durationInFrames / composicao.fps,
  });
}

function lerEntrada() {
  return new Promise((resolve, reject) => {
    let buf = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (c) => (buf += c));
    process.stdin.on("end", () => resolve(buf.trim()));
    process.stdin.on("error", reject);
  });
}

main().catch((e) => {
  // Toda falha sai pelo mesmo envelope. Sem isto o Rust receberia stdout vazio
  // e teria que adivinhar a causa no stack trace do stderr.
  emitir({ ok: false, erro: String(e?.message ?? e) });
  process.exitCode = 1;
});
