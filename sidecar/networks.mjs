// Adaptadores por rede social.
//
// Seletor de rede social quebra. Isso nao e pessimismo, e o estado do mundo:
// essas paginas mudam markup toda semana e algumas ativamente resistem a
// automacao. Por isso cada acao aqui tenta uma lista de seletores em ordem, do
// mais estavel (papel ARIA, texto visivel em pt e en) para o mais fragil
// (classe gerada). Quando nada funciona, o adaptador falha com mensagem legivel
// em vez de clicar no lugar errado.

/** Tenta varios seletores e devolve o primeiro localizador visivel. */
export async function firstVisible(scope, selectors, timeout = 8000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    for (const selector of selectors) {
      const locator = scope.locator(selector).first();
      try {
        if (await locator.isVisible({ timeout: 250 })) return locator;
      } catch {
        // seletor invalido ou ainda nao montado: segue para o proximo
      }
    }
    await scope.waitForTimeout(200);
  }
  return null;
}

async function clickAny(scope, selectors, label, timeout = 8000) {
  const locator = await firstVisible(scope, selectors, timeout);
  if (!locator) throw new Error(`nao encontrei o elemento: ${label}`);
  await locator.click();
  return locator;
}

async function typeInto(scope, selectors, text, label) {
  const locator = await firstVisible(scope, selectors, 10000);
  if (!locator) throw new Error(`nao encontrei o campo: ${label}`);
  await locator.click();
  await locator.type(text, { delay: 12 });
  return locator;
}

/** Coleta texto de varios cartoes para virar observacao de campo. */
async function harvest(page, selectors, limit) {
  const out = [];
  for (const selector of selectors) {
    const nodes = await page.locator(selector).all().catch(() => []);
    for (const node of nodes) {
      if (out.length >= limit) break;
      const text = (await node.innerText().catch(() => "")).trim().replace(/\s+/g, " ");
      if (text.length > 25) out.push(text.slice(0, 400));
    }
    if (out.length >= limit) break;
  }
  return out;
}

// ------------------------------------------------------------------ Instagram

const instagram = {
  home: "https://www.instagram.com/",
  async isLoggedIn(page) {
    const marker = await firstVisible(
      page,
      ['svg[aria-label="Home"]', 'svg[aria-label="Página inicial"]', 'a[href="/direct/inbox/"]'],
      5000
    );
    return marker !== null;
  },
  async login(page, username, password) {
    await page.goto("https://www.instagram.com/accounts/login/", { waitUntil: "domcontentloaded" });
    await typeInto(page, ['input[name="username"]'], username, "usuario");
    await typeInto(page, ['input[name="password"]'], password, "senha");
    await clickAny(page, ['button[type="submit"]'], "botao entrar");
    // 2FA e checkpoint exigem a pessoa. Damos tempo de janela aberta para isso.
    await page.waitForTimeout(6000);
    return await instagram.isLoggedIn(page);
  },
  async metrics(page, limit) {
    // O proprio perfil: o link do avatar leva a conta logada sem precisar
    // saber o @ dela.
    await page.goto("https://www.instagram.com/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(2500);
    const perfil = await firstVisible(
      page,
      ['a[href^="/"][role="link"]:has(img[alt*="foto do perfil" i])', 'a:has(> img[alt*="profile photo" i])'],
      6000
    );
    if (perfil) {
      await perfil.click().catch(() => {});
      await page.waitForTimeout(3000);
    }
    const links = await linksDePosts(page, "^/(p|reel)/", limit);
    const out = [];
    for (const url of links) {
      await page.goto(url, { waitUntil: "domcontentloaded" }).catch(() => {});
      await page.waitForTimeout(2200);
      const n = await numerosDaPagina(
        page,
        ['section:has(svg[aria-label="Like"]) span:has-text("curtida")', 'a[href$="/liked_by/"] span', 'span:has-text("curtidas")', 'span:has-text("likes")'],
        ['a[href$="/comments/"] span', 'span:has-text("comentarios")', 'span:has-text("comments")']
      );
      const resumo = await page.locator("h1, article span").first().innerText().catch(() => "");
      out.push({ url, resumo: (resumo || "").slice(0, 120), publicado_em: await dataDoPost(page), ...n });
    }
    return out;
  },
  async research(page, limit) {
    await page.goto("https://www.instagram.com/explore/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(3500);
    const explore = await harvest(page, ["article", 'div[role="button"] span'], limit);
    await page.goto("https://www.instagram.com/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(3000);
    const feed = await harvest(page, ["article"], limit);
    return [
      "EXPLORAR (o que o algoritmo esta empurrando agora):",
      ...explore.map((t) => `- ${t}`),
      "",
      "FEED DA CONTA (quem ela segue e o que publica):",
      ...feed.map((t) => `- ${t}`),
    ].join("\n");
  },
  async publish(page, { imagePath, caption, dryRun }) {
    await page.goto("https://www.instagram.com/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(2500);

    await clickAny(
      page,
      [
        'svg[aria-label="New post"]',
        'svg[aria-label="Nova publicação"]',
        'a[href="#"]:has(svg[aria-label="New post"])',
        'div[role="button"]:has-text("Create")',
        'div[role="button"]:has-text("Criar")',
      ],
      "botao de nova publicacao"
    );
    await page.waitForTimeout(1200);

    // O menu de criar pode abrir um submenu com "Publicação".
    const submenu = await firstVisible(page, ['div[role="menuitem"]:has-text("Post")', 'div[role="menuitem"]:has-text("Publicação")'], 2500);
    if (submenu) {
      await submenu.click();
      await page.waitForTimeout(1000);
    }

    const input = page.locator('input[type="file"]').first();
    await input.waitFor({ state: "attached", timeout: 15000 });
    await input.setInputFiles(imagePath);
    await page.waitForTimeout(2500);

    // Corte e edicao: dois "Avançar".
    for (let i = 0; i < 2; i++) {
      const next = await firstVisible(page, ['div[role="button"]:has-text("Next")', 'div[role="button"]:has-text("Avançar")', 'button:has-text("Next")'], 8000);
      if (!next) break;
      await next.click();
      await page.waitForTimeout(1800);
    }

    await typeInto(
      page,
      ['textarea[aria-label="Write a caption..."]', 'textarea[aria-label="Escreva uma legenda..."]', 'div[contenteditable="true"][role="textbox"]'],
      caption,
      "legenda"
    );

    if (dryRun) {
      return { published: false, detail: "Simulacao: parou antes de compartilhar, com tudo preenchido." };
    }
    await clickAny(page, ['div[role="button"]:has-text("Share")', 'div[role="button"]:has-text("Compartilhar")'], "botao compartilhar");
    await page.waitForTimeout(6000);
    return { published: true, detail: "Publicacao enviada ao Instagram." };
  },
};

// ------------------------------------------------------------------- Facebook

const facebook = {
  home: "https://www.facebook.com/",
  async isLoggedIn(page) {
    return (await firstVisible(page, ['div[aria-label="Create a post"]', 'div[aria-label="Criar publicação"]', '[aria-label="Your profile"]'], 5000)) !== null;
  },
  async login(page, username, password) {
    await page.goto("https://www.facebook.com/login", { waitUntil: "domcontentloaded" });
    await typeInto(page, ["#email", 'input[name="email"]'], username, "email");
    await typeInto(page, ["#pass", 'input[name="pass"]'], password, "senha");
    await clickAny(page, ['button[name="login"]', 'button[type="submit"]'], "botao entrar");
    await page.waitForTimeout(6000);
    return await facebook.isLoggedIn(page);
  },
  async metrics(page, limit) {
    await page.goto("https://www.facebook.com/me", { waitUntil: "domcontentloaded" }).catch(() => {});
    await page.waitForTimeout(3000);
    const links = await linksDePosts(page, "/(posts|permalink)/", limit);
    const out = [];
    for (const url of links) {
      await page.goto(url, { waitUntil: "domcontentloaded" }).catch(() => {});
      await page.waitForTimeout(2200);
      const n = await numerosDaPagina(
        page,
        ['div[aria-label*="reac" i]', 'span[aria-label*="curtida" i]', 'span[aria-label*="like" i]'],
        ['span:has-text("comentario")', 'span:has-text("comment")']
      );
      const resumo = await page.locator('div[data-ad-preview="message"], div[role="article"]').first().innerText().catch(() => "");
      out.push({ url, resumo: (resumo || "").slice(0, 120), publicado_em: await dataDoPost(page), ...n });
    }
    return out;
  },
  async research(page, limit) {
    await page.goto("https://www.facebook.com/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(4000);
    const posts = await harvest(page, ['div[role="article"]'], limit);
    return ["FEED DO FACEBOOK:", ...posts.map((t) => `- ${t}`)].join("\n");
  },
  async publish(page, { imagePath, caption, dryRun }) {
    await page.goto("https://www.facebook.com/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(3000);
    await clickAny(
      page,
      ['div[role="button"]:has-text("What\'s on your mind")', 'div[role="button"]:has-text("No que você está pensando")', 'div[aria-label="Create a post"]'],
      "caixa de criar publicacao"
    );
    await page.waitForTimeout(2000);

    const dialog = page.locator('div[role="dialog"]').last();
    await typeInto(dialog, ['div[contenteditable="true"][role="textbox"]'], caption, "texto da publicacao");

    const photoButton = await firstVisible(dialog, ['div[aria-label="Photo/video"]', 'div[aria-label="Foto/vídeo"]'], 5000);
    if (photoButton) {
      await photoButton.click();
      await page.waitForTimeout(1200);
    }
    const input = dialog.locator('input[type="file"]').first();
    await input.waitFor({ state: "attached", timeout: 12000 });
    await input.setInputFiles(imagePath);
    await page.waitForTimeout(3000);

    if (dryRun) {
      return { published: false, detail: "Simulacao: composer preenchido, sem publicar." };
    }
    await clickAny(dialog, ['div[aria-label="Post"]', 'div[aria-label="Publicar"]', 'div[role="button"]:has-text("Post")'], "botao publicar");
    await page.waitForTimeout(6000);
    return { published: true, detail: "Publicacao enviada ao Facebook." };
  },
};

// --------------------------------------------------------------------- TikTok

const tiktok = {
  home: "https://www.tiktok.com/",
  async isLoggedIn(page) {
    return (await firstVisible(page, ['[data-e2e="profile-icon"]', 'a[href*="/upload"]'], 5000)) !== null;
  },
  async login(page, username, password) {
    await page.goto("https://www.tiktok.com/login/phone-or-email/email", { waitUntil: "domcontentloaded" });
    await typeInto(page, ['input[name="username"]'], username, "usuario");
    await typeInto(page, ['input[type="password"]'], password, "senha");
    await clickAny(page, ['button[data-e2e="login-button"]', 'button[type="submit"]'], "botao entrar");
    // O TikTok quase sempre cai em captcha. A janela fica aberta para a pessoa resolver.
    await page.waitForTimeout(15000);
    return await tiktok.isLoggedIn(page);
  },
  async metrics(page, limit) {
    await page.goto("https://www.tiktok.com/profile", { waitUntil: "domcontentloaded" }).catch(() => {});
    await page.waitForTimeout(3000);
    const links = await linksDePosts(page, "/video/", limit);
    const out = [];
    for (const url of links) {
      await page.goto(url, { waitUntil: "domcontentloaded" }).catch(() => {});
      await page.waitForTimeout(2500);
      const n = await numerosDaPagina(
        page,
        ['[data-e2e="like-count"]', '[data-e2e="browse-like-count"]'],
        ['[data-e2e="comment-count"]', '[data-e2e="browse-comment-count"]']
      );
      const resumo = await page.locator('[data-e2e="browse-video-desc"], h1').first().innerText().catch(() => "");
      out.push({ url, resumo: (resumo || "").slice(0, 120), publicado_em: await dataDoPost(page), ...n });
    }
    return out;
  },
  async research(page, limit) {
    await page.goto("https://www.tiktok.com/explore", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(4000);
    const items = await harvest(page, ['[data-e2e="explore-item"]', 'div[data-e2e="recommend-list-item-container"]'], limit);
    return ["EXPLORAR DO TIKTOK:", ...items.map((t) => `- ${t}`)].join("\n");
  },
  async publish(page, { imagePath, caption, dryRun }) {
    await page.goto("https://www.tiktok.com/tiktokstudio/upload", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(5000);
    const input = page.locator('input[type="file"]').first();
    await input.waitFor({ state: "attached", timeout: 20000 });
    await input.setInputFiles(imagePath);
    await page.waitForTimeout(6000);

    await typeInto(page, ['div[contenteditable="true"]', 'div[data-e2e="caption-input"]'], caption, "legenda");

    if (dryRun) {
      return { published: false, detail: "Simulacao: upload preenchido, sem postar. O TikTok espera video; imagem pode ser recusada." };
    }
    await clickAny(page, ['button:has-text("Post")', 'button:has-text("Publicar")'], "botao postar", 15000);
    await page.waitForTimeout(10000);
    return { published: true, detail: "Envio ao TikTok concluido." };
  },
};

// ------------------------------------------------------------------- LinkedIn

const linkedin = {
  home: "https://www.linkedin.com/feed/",
  async isLoggedIn(page) {
    return (await firstVisible(page, ['button:has-text("Start a post")', 'button:has-text("Comece uma publicação")', ".share-box-feed-entry__trigger"], 5000)) !== null;
  },
  async login(page, username, password) {
    await page.goto("https://www.linkedin.com/login", { waitUntil: "domcontentloaded" });
    await typeInto(page, ["#username"], username, "usuario");
    await typeInto(page, ["#password"], password, "senha");
    await clickAny(page, ['button[type="submit"]'], "botao entrar");
    await page.waitForTimeout(6000);
    return await linkedin.isLoggedIn(page);
  },
  async metrics(page, limit) {
    // A propria atividade recente, que e onde as publicacoes ficam listadas.
    await page.goto("https://www.linkedin.com/in/me/recent-activity/all/", { waitUntil: "domcontentloaded" }).catch(() => {});
    await page.waitForTimeout(3200);
    const cartoes = page.locator("div.feed-shared-update-v2, article");
    const total = Math.min(await cartoes.count().catch(() => 0), limit);
    const out = [];
    for (let i = 0; i < total; i++) {
      const c = cartoes.nth(i);
      const texto = (await c.innerText().catch(() => "")) || "";
      // O LinkedIn mostra os contadores no proprio cartao, entao nao ha
      // navegacao peca a peca: menos cliques, menos chance de quebrar.
      const curtidas = paraNumero((texto.match(/([\d.,]+\s*(mil|K|M)?)\s*(reac|curtid|like)/i) || [])[1]);
      const comentarios = paraNumero((texto.match(/([\d.,]+\s*(mil|K|M)?)\s*(coment)/i) || [])[1]);
      const url = await c.locator("a[href*='/feed/update/']").first().getAttribute("href").catch(() => null);
      out.push({
        url: url ? new URL(url, "https://www.linkedin.com").href : `linkedin-atividade-${i}`,
        resumo: texto.split("\n").filter(Boolean).slice(1, 3).join(" ").slice(0, 120),
        publicado_em: "",
        curtidas,
        comentarios,
      });
    }
    return out;
  },
  async research(page, limit) {
    await page.goto("https://www.linkedin.com/feed/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(4000);
    const posts = await harvest(page, ["div.feed-shared-update-v2", 'div[data-id^="urn:li:activity"]'], limit);
    return ["FEED DO LINKEDIN:", ...posts.map((t) => `- ${t}`)].join("\n");
  },
  async publish(page, { imagePath, caption, dryRun }) {
    await page.goto("https://www.linkedin.com/feed/", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(3000);
    await clickAny(page, ['button:has-text("Start a post")', 'button:has-text("Comece uma publicação")', ".share-box-feed-entry__trigger"], "botao de criar publicacao");
    await page.waitForTimeout(2500);

    const dialog = page.locator('div[role="dialog"]').last();
    const media = await firstVisible(dialog, ['button[aria-label="Add media"]', 'button[aria-label="Adicionar mídia"]'], 5000);
    if (media) {
      await media.click();
      await page.waitForTimeout(1500);
    }
    const input = page.locator('input[type="file"]').first();
    await input.waitFor({ state: "attached", timeout: 12000 });
    await input.setInputFiles(imagePath);
    await page.waitForTimeout(3500);

    const next = await firstVisible(page, ['button:has-text("Next")', 'button:has-text("Avançar")'], 6000);
    if (next) {
      await next.click();
      await page.waitForTimeout(2000);
    }
    await typeInto(page, ['div[role="textbox"]', ".ql-editor"], caption, "texto da publicacao");

    if (dryRun) {
      return { published: false, detail: "Simulacao: publicacao montada, sem publicar." };
    }
    await clickAny(page, ['button:has-text("Post")', 'button:has-text("Publicar")'], "botao publicar");
    await page.waitForTimeout(6000);
    return { published: true, detail: "Publicacao enviada ao LinkedIn." };
  },
};

// -------------------------------------------------------------------------- X

const x = {
  home: "https://x.com/home",
  async isLoggedIn(page) {
    return (await firstVisible(page, ['a[data-testid="SideNav_NewTweet_Button"]', '[data-testid="tweetTextarea_0"]'], 5000)) !== null;
  },
  async login(page, username, password) {
    await page.goto("https://x.com/i/flow/login", { waitUntil: "domcontentloaded" });
    await typeInto(page, ['input[autocomplete="username"]'], username, "usuario");
    await clickAny(page, ['button:has-text("Next")', 'button:has-text("Avançar")'], "avancar");
    await page.waitForTimeout(2500);
    await typeInto(page, ['input[name="password"]'], password, "senha");
    await clickAny(page, ['button[data-testid="LoginForm_Login_Button"]', 'button:has-text("Log in")'], "entrar");
    await page.waitForTimeout(6000);
    return await x.isLoggedIn(page);
  },
  async metrics(page, limit) {
    await page.goto("https://x.com/home", { waitUntil: "domcontentloaded" }).catch(() => {});
    await page.waitForTimeout(2500);
    const perfil = await firstVisible(page, ['a[data-testid="AppTabBar_Profile_Link"]'], 6000);
    if (perfil) {
      await perfil.click().catch(() => {});
      await page.waitForTimeout(3000);
    }
    const cartoes = page.locator('article[data-testid="tweet"]');
    const total = Math.min(await cartoes.count().catch(() => 0), limit);
    const out = [];
    for (let i = 0; i < total; i++) {
      const c = cartoes.nth(i);
      const conta = async (testid) =>
        paraNumero((await c.locator(`[data-testid="${testid}"]`).first().innerText().catch(() => "")) || "");
      const url = await c.locator('a[href*="/status/"]').first().getAttribute("href").catch(() => null);
      const texto = (await c.locator('[data-testid="tweetText"]').first().innerText().catch(() => "")) || "";
      out.push({
        url: url ? new URL(url, "https://x.com").href : `x-post-${i}`,
        resumo: texto.slice(0, 120),
        publicado_em: (await c.locator("time").first().getAttribute("datetime").catch(() => "")) || "",
        curtidas: await conta("like"),
        comentarios: await conta("reply"),
        // O X mostra impressao no proprio cartao, e e a unica rede que faz isso.
        impressoes: paraNumero(
          (await c.locator('a[href$="/analytics"], [aria-label*="View" i]').first().innerText().catch(() => "")) || ""
        ),
      });
    }
    return out;
  },
  async research(page, limit) {
    await page.goto("https://x.com/explore", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(4000);
    const items = await harvest(page, ['[data-testid="trend"]', "article"], limit);
    return ["EXPLORAR DO X (assuntos em alta):", ...items.map((t) => `- ${t}`)].join("\n");
  },
  async publish(page, { imagePath, caption, dryRun }) {
    await page.goto("https://x.com/compose/post", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(3000);
    await typeInto(page, ['[data-testid="tweetTextarea_0"]', 'div[role="textbox"]'], caption.slice(0, 280), "texto do post");
    const input = page.locator('input[type="file"]').first();
    await input.waitFor({ state: "attached", timeout: 10000 });
    await input.setInputFiles(imagePath);
    await page.waitForTimeout(4000);

    if (dryRun) {
      return { published: false, detail: "Simulacao: post montado, sem enviar." };
    }
    await clickAny(page, ['[data-testid="tweetButton"]', 'button:has-text("Post")'], "botao postar");
    await page.waitForTimeout(5000);
    return { published: true, detail: "Post enviado ao X." };
  },
};


export { paraNumero } from "./metricas.mjs";

export const NETWORKS = { instagram, facebook, tiktok, linkedin, x };
