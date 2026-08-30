#!/usr/bin/env node
// Sidecar do Playwright.
//
// Protocolo: uma requisicao JSON por linha no stdin, uma resposta JSON por linha
// no stdout. Todo log de diagnostico vai para stderr, para nunca poluir o canal
// de resposta que o Rust esta lendo.
//
// Um contexto persistente por rede social, guardado em disco. E isso que faz o
// login acontecer uma vez so: a sessao sobrevive ao fechamento do app, e nas
// execucoes seguintes o agente ja entra logado sem a senha passar por lugar nenhum.

import { chromium } from "playwright";
import path from "node:path";
import fs from "node:fs";
import readline from "node:readline";
import { NETWORKS } from "./networks.mjs";

const PROFILES_DIR = process.env.POSTLY_PROFILES || path.join(process.cwd(), ".profiles");
const contexts = new Map(); // slug -> { context, page }

const log = (...args) => console.error("[sidecar]", ...args);

function reply(id, ok, data, error) {
  process.stdout.write(JSON.stringify({ id, ok, data: data ?? {}, error }) + "\n");
}

function adapter(slug) {
  const found = NETWORKS[slug];
  if (!found) throw new Error(`rede desconhecida: ${slug}`);
  return found;
}

async function contextFor(slug, headless = false) {
  if (contexts.has(slug)) return contexts.get(slug);

  const profileDir = path.join(PROFILES_DIR, slug);
  fs.mkdirSync(profileDir, { recursive: true });

  const context = await chromium.launchPersistentContext(profileDir, {
    headless,
    viewport: { width: 1366, height: 900 },
    locale: "pt-BR",
    timezoneId: "America/Sao_Paulo",
    args: [
      // Reduz o consumo do Chromium, que divide RAM com o modelo local.
      "--disable-dev-shm-usage",
      "--disable-background-networking",
      "--disable-features=Translate,MediaRouter",
      "--renderer-process-limit=2",
    ],
  });

  // Remove o marcador mais obvio de automacao. Isto e higiene de compatibilidade
  // com paginas que quebram quando detectam webdriver, nao evasao de deteccao:
  // o login continua sendo o do proprio usuario, na sessao dele.
  await context.addInitScript(() => {
    Object.defineProperty(navigator, "webdriver", { get: () => undefined });
  });

  const page = context.pages()[0] || (await context.newPage());
  page.setDefaultTimeout(30000);

  const entry = { context, page };
  contexts.set(slug, entry);
  return entry;
}

async function shot(slug, page, tag) {
  try {
    const dir = path.join(PROFILES_DIR, "_screenshots");
    fs.mkdirSync(dir, { recursive: true });
    const file = path.join(dir, `${slug}-${tag}-${Date.now()}.png`);
    await page.screenshot({ path: file, fullPage: false });
    return file;
  } catch {
    return null;
  }
}

const HANDLERS = {
  async open({ network, url, headless }) {
    const net = adapter(network);
    const { page } = await contextFor(network, headless === true);
    await page.goto(url || net.home, { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(2500);
    const loggedIn = await net.isLoggedIn(page);
    return { loggedIn, url: page.url() };
  },

  async login({ network, username, password }) {
    const net = adapter(network);
    const { page } = await contextFor(network, false);
    if (await net.isLoggedIn(page)) {
      return { loggedIn: true, url: page.url(), detail: "sessao ja estava valida no perfil" };
    }
    if (!username || !password) {
      throw new Error("sem credenciais e sem sessao salva: faca login na janela aberta e tente de novo");
    }
    const loggedIn = await net.login(page, username, password);
    return { loggedIn, url: page.url() };
  },

  async research({ network, limit }) {
    const net = adapter(network);
    const { page } = await contextFor(network, false);
    if (!(await net.isLoggedIn(page))) {
      return { report: "Sem sessao ativa nesta rede; nao foi possivel observar o feed." };
    }
    const report = await net.research(page, Math.max(1, Math.min(limit || 8, 25)));
    return { report };
  },

  async metrics({ network, limit }) {
    const net = adapter(network);
    if (typeof net.metrics !== "function") return { posts: [] };
    const { page } = await contextFor(network, false);
    if (!(await net.isLoggedIn(page))) {
      throw new Error("sem sessao ativa nesta rede; faca login antes de coletar");
    }
    // Uma falha de seletor devolve lista vazia, nunca numero chutado: numero
    // errado envenena a mediana, e a mediana e o que decide a proxima campanha.
    try {
      const posts = await net.metrics(page, Math.max(1, Math.min(limit || 8, 25)));
      return { posts: posts.filter((p) => p && (p.curtidas || p.comentarios)) };
    } catch (err) {
      log("metrics", "falhou:", err.message);
      return { posts: [] };
    }
  },

  async publish({ network, imagePath, caption, dryRun }) {
    const net = adapter(network);
    if (!imagePath || !fs.existsSync(imagePath)) {
      throw new Error(`imagem nao encontrada: ${imagePath}`);
    }
    const { page } = await contextFor(network, false);
    if (!(await net.isLoggedIn(page))) {
      throw new Error("sem sessao ativa nesta rede; rode o login antes de publicar");
    }
    try {
      const result = await net.publish(page, { imagePath, caption, dryRun: dryRun === true });
      const screenshot = await shot(network, page, dryRun ? "simulacao" : "publicado");
      return { ...result, screenshot };
    } catch (err) {
      const screenshot = await shot(network, page, "erro");
      const hint = screenshot ? ` Captura do estado da pagina em ${screenshot}.` : "";
      throw new Error(`${err.message}.${hint} A janela ficou aberta para voce concluir na mao.`);
    }
  },

  async close({ network }) {
    const entry = contexts.get(network);
    if (entry) {
      await entry.context.close().catch(() => {});
      contexts.delete(network);
    }
    return { closed: true };
  },

  async shutdown() {
    for (const [, entry] of contexts) {
      await entry.context.close().catch(() => {});
    }
    contexts.clear();
    // Sai no proximo tick, depois de `reply` ter escrito a resposta no stdout.
    setImmediate(() => setTimeout(() => process.exit(0), 50));
    return { closed: true };
  },
};

const rl = readline.createInterface({ input: process.stdin, terminal: false });

// Uma requisicao por vez, na ordem de chegada.
//
// `rl.on("line", async ...)` NAO espera o handler anterior: duas linhas que
// chegam juntas rodam concorrentes, e um `shutdown` pode ultrapassar um `open`
// que ainda esta subindo o navegador. Encadear as promessas numa fila resolve
// isso na origem, em vez de depender de quem chama se comportar.
let fila = Promise.resolve();

async function processar(raw) {
  let request;
  try {
    request = JSON.parse(raw);
  } catch {
    log("linha ilegivel ignorada");
    return;
  }
  const { id, cmd, ...payload } = request;
  const handler = HANDLERS[cmd];
  if (!handler) {
    reply(id, false, null, `comando desconhecido: ${cmd}`);
    return;
  }
  try {
    const data = await handler(payload);
    reply(id, true, data);
  } catch (err) {
    log(cmd, "falhou:", err.message);
    reply(id, false, null, err.message);
  }
}

rl.on("line", (line) => {
  const raw = line.trim();
  if (!raw) return;
  fila = fila.then(() => processar(raw));
});

// Fim do stdin: espera a fila drenar antes de sair. Sem isto, um EOF que chega
// junto com a ultima requisicao descarta trabalho ja aceito.
rl.on("close", () => {
  fila = fila.then(async () => {
    for (const [, entry] of contexts) {
      await entry.context.close().catch(() => {});
    }
    process.exit(0);
  });
});

log("pronto, perfis em", PROFILES_DIR);
