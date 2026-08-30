// The site text in English.
//
// Not a literal translation of `pt.ts`: the Portuguese voice is plain and
// short, and keeping that in English means rewriting sentences rather than
// mapping them word by word.

import type { Dicionario } from "./pt";

export const en: Dicionario = {
  meta: {
    lang: "en",
    titulo: "Postly — a marketing department that runs on your machine",
    descricao:
      "Open source desktop app that orchestrates local AI models as a marketing team. Four roles take turns to create, audit and publish to your social networks. One model at a time, no subscription.",
    ogDescricao:
      "A marketing department that runs on your machine. Open source, non-profit.",
  },

  nav: {
    como: "How it works",
    video: "Video",
    telas: "Screens",
    claude: "Claude Code",
    instalacao: "Install",
    github: "GitHub",
    idioma: "Language",
  },

  heroi: {
    pilula: "open source, non-profit",
    titulo1: "A marketing department",
    titulo2: "that runs on your machine.",
    linha:
      "Four AI roles take turns on local models to research the market, create the post, audit it and publish to your networks. One model at a time, because four of them don't fit on an ordinary machine.",
    verComo: "See how it works",
    lerCodigo: "Read the code",
  },

  faixa: "open source · non-profit · runs on your machine",

  cargos: [
    { cargo: "Director", nota: "Decides the strategy across networks. Only exists when there is more than one." },
    { cargo: "Sector Manager", nota: "One per network. Reads the market through the browser and sets the creative line." },
    { cargo: "Creator", nota: "Gets a closed brief and produces. Decides nothing, so it doesn't need expensive reasoning." },
    { cargo: "Auditor", nota: "Judges the finished post with the image in front of it. A rejection goes back to the Creator." },
  ],
  trilhaRotulo: (postas: string) => `Route: ${postas}`,
  trilhaEntao: ", then ",

  video: {
    titulo: "Under a minute",
    texto:
      "The whole project, from the problem to the install command: the relay, the hardware inversion, the integrations, and what never leaves your machine.",
    assistir: "Watch the 48-second narrated walkthrough",
    duracao: "48 s",
  },

  porque: {
    titulo: "Why this exists",
    texto:
      "AI marketing tools charge by the month, keep your campaigns on someone else's server, and hide which model wrote what.",
    p1: "Postly does the opposite: the models run on your hardware, every conversation stays as Markdown in a file you own, and the only variable cost is image generation.",
    p2: "The hard part was never calling a model. It was fitting four of them on an ordinary machine, and that is what the middleware behind the relay is about.",
  },

  como: {
    titulo: "One model at a time",
    texto:
      "A marketing agent needs reasoning to decide and obedience to execute, and those are different models. Running them all at once does not fit.",
    passos: [
      {
        titulo: "Measures the memory",
        texto: "Before every turn the system reads how much RAM is free at that moment, not how much the machine has.",
      },
      {
        titulo: "Picks the strongest that fits",
        texto: "Within that role's tier. If nothing fits, it drops the tier and says so in the report instead of failing.",
      },
      {
        titulo: "Records the whole conversation",
        texto: "System prompt, input, full answer and the reasoning, one file per turn. That is what makes an audit possible later.",
      },
      {
        titulo: "Unloads and hands over",
        texto: "Only the message that crosses goes to the next role. There are never two models resident at once.",
      },
    ],
  },

  duelo: {
    titulo: "The catalog ranks by speed, not by size",
    texto:
      "The choice changes with the hardware, and on a PC without a GPU it inverts the intuition of anyone looking only at file size.",
    denso: "dense · 14B active",
    moe: "MoE · 3B active of 30B",
    discoA: "9.3 GB on disk",
    discoB: "19 GB on disk",
    nota:
      "Measured on this development machine: Ryzen 5000, no usable GPU. The model that takes twice the memory generates almost ten times faster, because only the active experts go through the CPU. Optimizing for file size leads to the wrong pick.",
  },

  telas: {
    titulo: "What you open",
    texto:
      "The application, running here on the page. Same React, same CSS — switch tabs, drag the graph nodes.",
    legendas: {
      modelos: "The catalog ranks by measured speed, not by file size. Downloading and removing happen from here.",
      campanha: "The relay shows where the dispatch is. Each turn takes minutes, so you can leave and come back.",
      cerebro: "Drag the nodes. The neighborhood on the right is exactly what an agent receives when it queries.",
    },
    avancar: "Advance the turn",
  },

  laco: {
    titulo: "Performance feeds back in",
    texto:
      "A content generator with no performance feedback repeats what the model finds pretty, not what worked.",
    p1: "You record the result of each post and the system ranks it against your own account's median. That reading goes into the prompt of the role that decides the next campaign, with the exact number to beat.",
    p2: "The rule is always beat it and never repeat it: what worked becomes the floor, not the mold. The one exception is the outlier, when a post hits three times the median — at that point it stopped being luck, and it is worth staying on that line while it pays.",
  },

  privacidade: {
    titulo: "What leaves your machine",
    sai: "leaves",
    fica: "stays",
    saiItens: [
      "The image prompt, to the art service you chose.",
      "Browser traffic to the networks you publish on, in your own session.",
    ],
    ficaItens: [
      "The models and everything they write.",
      "The context graph, the campaigns and the transcripts.",
      "The credentials, encrypted with AES-256-GCM.",
    ],
    nota:
      "There is no telemetry, no project server, no account to create. About the vault, a caveat usually sold with more confidence than it deserves: it protects against a backup, a cloud sync and someone reading the disk. It does not protect against a program running as your user.",
  },

  claude: {
    pilula: "optional integration",
    titulo: "Or let Claude Code run the turns",
    texto:
      "A local model on a CPU delivers close to 1 token per second, and a full campaign takes the afternoon. If you already subscribe to Claude Code, you can point all four roles at it and the same campaign finishes in minutes.",
    itens: [
      {
        titulo: "The role's tier still decides",
        texto:
          "Only the axis changes: whoever decides and judges goes to Opus, whoever audits within a given criterion goes to Sonnet, and whoever executes a closed brief goes to Haiku. The relay is the same.",
      },
      {
        titulo: "On your subscription, not per token",
        texto:
          "Postly runs the `claude` binary already on your machine, with the session you already signed into. There is no API key field anywhere in the app, and the vault has never held one.",
      },
      {
        titulo: "Tools stay off",
        texto:
          "Bash, file reads and writes, web search: none of it is offered to the role. It writes text and hands it back. A marketing agent has no business touching your disk.",
      },
    ],
    nota:
      "One trap worth naming: the CLI prefers ANTHROPIC_API_KEY when it is present in the environment, and a child process inherits the parent's environment. Anyone with that variable exported in their shell would start paying per token without noticing. Postly strips it from the child process, along with the Bedrock and Vertex credentials. The turn runs on the subscription, or it does not run.",
    ressalva:
      "Claude and Claude Code are trademarks of Anthropic. Postly is an independent project, not affiliated with Anthropic.",
  },

  instalacao: {
    titulo: "Install",
    texto:
      "One command. It detects your system, confirms with you and downloads the latest release.",
    requisitos: [
      {
        titulo: "Ollama",
        texto:
          "You don't need to install it first. The app does it on first launch, along with the browser that publishes, with a progress bar.",
      },
      {
        titulo: "One image key",
        texto:
          "Gemini, OpenAI, FLUX, Stability AI or Higgsfield. Only the art needs an external service; all the text comes from the local models.",
      },
      {
        titulo: "Memory",
        texto:
          "16 GB for comfortable use. It runs on 8 GB with smaller models, and the catalog shows which ones fit before you download.",
      },
    ],
  },

  comando: {
    rotulo: "Operating system",
    ouBaixe: "Or download the installer:",
    emBreve: "under review",
    notaWinget: "The package is in the winget-pkgs queue. This command starts working the moment it is approved.",
    msi: "installs for everyone, asks for admin",
    exe: "installs just for you, no admin",
    copiar: "copy",
    copiado: "copied",
    notas: {
      linux: "Installs an AppImage into ~/.local/bin. Runs on any distro and doesn't ask for root.",
      macos: "Same command. On first launch, right-click the app and choose Open.",
      windows: "Paste it into Git Bash, which ships with Git for Windows. The script detects your system and downloads the right installer.",
    },
  },

  aberto: {
    titulo: "Non-profit",
    p1: "Postly is free and MIT licensed. There is no paid tier, plan or subscription, and no intention of creating one. The entire codebase is on GitHub, including the vault that holds your credentials, because an app that asks for an API key has to be auditable.",
    p2: "If any of this was useful, the feedback that makes a difference is an issue with a reproducible problem, or a social network adapter that stopped working.",
    ver: "View on GitHub",
    issue: "Open an issue",
  },

  rodape: { nota: "MIT · built by" },

  vitrine: {
    abas: { modelos: "Models", campanha: "Campaign", cerebro: "Brain" },
    ram: "21.5 GB free",

    modelos: {
      titulo: "What runs here",
      texto: "You don't choose. For each role, the strongest model that fits comes up.",
      teto: "ceiling per model",
      tetoNota: "CPU only",
      cabe: "fit",
      cabeNota: "1 out of reach for this hardware",
      baixado: "downloaded",
      todas: "All",
      avancado: "Advanced setup",
      decisao: "Decision",
      decisaoNota: "Director and Manager. They choose the language and judge the post.",
      tagBaixado: "downloaded",
      tagNaoCabe: "doesn't fit now",
      tagVisao: "sees images",
      remover: "Remove",
      baixar: "Download",
      denso: (p: string) => `dense · ${p}B active`,
      moe: (a: string, p: number) => `MoE · ${a}B active of ${p}B`,
      notas: {
        "qwen3:30b-a3b": "Only the active experts go through the CPU — that is why it is the fastest here.",
        "gpt-oss:20b": "Strong reasoning on a modest footprint. Good for the role that decides.",
        "llama3.3:70b": "Dense and expensive. Only makes sense with plenty of VRAM.",
        "gemma3:4b": "Executes a closed brief without inventing. Sees images.",
        "deepseek-r1:14b": "Thinks before answering. Serves the auditor well.",
        "mistral-small:24b": "Fits on disk, but not in free memory right now.",
        "phi-4:14b": "Small and obedient. Good cost per token.",
        "granite3.3:8b": "Permissive license and predictable output.",
      },
      razoes: {
        "llama3.3:70b": "Doesn't fit: needs 45.2 GB and there are 21.5 GB free.",
      },
    },

    campanha: {
      titulo: "Campaign",
      texto: "Write what you want to achieve. The rest is the relay.",
      trilha: "Relay route",
      concluida: "done",
      turno: (i: number, n: number) => `turn ${i} of ${n}`,
      objetivo: "Goal",
      objetivoRotulo: "What you want to achieve",
      objetivoTexto:
        "Introduce the new roast to people who already buy whole bean and complain about acidity.",
      redes: "Networks",
      instagramNota: "square image, short caption",
      linkedinNota: "long text, cut at 210 characters",
      previsao: "What will happen",
      turnos: "agent turns",
      imagens: "images",
      imagensNota: "by Gemini",
      tempo: "estimated time",
      min: "min",
      rodar: "Run campaign",
      rodape:
        "Estimated from the speed measured on this machine. The default is Simulate: it builds the whole post and stops before the last click.",
      postas: [
        { cargo: "Director", nota: "sets the line" },
        { cargo: "Sector Manager", nota: "Instagram" },
        { cargo: "Content Creator", nota: "executes the brief" },
        { cargo: "Auditor", nota: "judges the post" },
      ],
    },

    cerebro: {
      titulo: "Brain",
      texto: "The context every role shares, as a weighted graph.",
      dica: "Drag a node to pin it. Scroll to zoom; double-click to reframe.",
      nenhum: "no node",
      vizinhanca: "neighborhood, ordered",
      explicacao:
        "This is exactly what an agent receives when it queries: already sorted by weight and cut at a threshold, so the cut does not happen inside the model.",
      nodes: {
        publico_alvo: "target_audience", tom_de_voz: "tone_of_voice", instagram: "instagram",
        linkedin: "linkedin", produto: "product", concorrente: "competitor",
        prova_social: "social_proof", objecao_preco: "price_objection",
        sazonalidade: "seasonality", formato_carrossel: "carousel_format",
        identidade_visual: "visual_identity", campanha_agosto: "august_campaign",
      },
      colNode: "node",
      colRelacao: "relation",
      colPeso: "weight",
      relacoes: {
        define: "defines", vive_em: "lives_in", levanta: "raises", combina: "matches",
        sustenta: "backs", compete: "competes", prefere: "prefers", rodou_em: "ran_on",
        considera: "considers", responde: "answers", restringe: "constrains",
        explora: "exploits", ajusta: "adjusts",
      },
    },
  },
};
