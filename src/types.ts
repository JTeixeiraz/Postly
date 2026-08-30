// Espelho tipado do que o Rust devolve. Mantido a mao de proposito: o contrato
// entre os dois lados e pequeno o bastante para nao valer um gerador.

export type Plataforma = "linux" | "windows" | "macos";
export type Pressao = "ok" | "apertado" | "critico";
export type Nivel = "alto" | "medio" | "baixo";

export interface RamSnapshot {
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  /** Teto para o catalogo: derivado da RAM que a maquina TEM. */
  max_budget_bytes: number;
  /** Teto para subir um agente agora: derivado da RAM livre. */
  live_budget_bytes: number;
  pressure: Pressao;
  low_ram_warning: boolean;
}

export interface OllamaStatus {
  state: "pronto" | "instalado_parado" | "ausente";
  version: string | null;
  binary_path: string | null;
  install_plan: Passo[];
}

export interface Passo {
  label: string;
  program: string;
  args: string[];
  needs_elevation: boolean;
}

export interface RedeInfo {
  slug: string;
  label: string;
  formato: string;
}

export type ModoComputacao = "dedicada" | "unificada" | "cpu";
export type Fabricante = "nvidia" | "amd" | "apple" | "intel" | "desconhecido";

export interface Acelerador {
  vendor: Fabricante;
  name: string;
  vram_total_bytes: number;
  vram_free_bytes: number;
  /** Memória unificada: a VRAM sai da mesma RAM do sistema. */
  unified: boolean;
  usable: boolean;
  detail: string;
}

export interface PerfilComputacao {
  ram: RamSnapshot;
  accelerators: Acelerador[];
  mode: ModoComputacao;
  mode_label: string;
  primary_name: string | null;
  vram_total_bytes: number;
  vram_free_bytes: number;
  /** O que a máquina carrega, mesmo devagar. */
  max_budget_bytes: number;
  /** O que cabe inteiro no acelerador. */
  accelerated_budget_bytes: number;
  /** O que cabe neste instante. */
  live_budget_bytes: number;
  prefers_moe: boolean;
  throughput_constant: number;
}

/** Andamento da instalacao do Ollama. */
export interface ProgressoProvisao {
  passo: number;
  total: number;
  label: string;
  linha: string;
  percent: number | null;
  fase: "instalando" | "subindo" | "fim";
}

export interface SondaSistema {
  plataforma: Plataforma;
  plataforma_label: string;
  node_instalado: boolean;
  diretorio_dados: string;
}

export interface Diagnostico {
  plataforma: Plataforma;
  plataforma_label: string;
  computacao: PerfilComputacao;
  ollama: OllamaStatus;
  node_instalado: boolean;
  diretorio_dados: string;
  redes_suportadas: RedeInfo[];
}

export interface AlvoLimpeza {
  label: string;
  path: string;
  bytes: number;
  safe: boolean;
}

export interface ProcessoPesado {
  pid: number;
  name: string;
  memory_bytes: number;
}

export interface PlanoOtimizacao {
  targets: AlvoLimpeza[];
  reclaimable_bytes: number;
  drop_caches: Passo | null;
  hogs: ProcessoPesado[];
}

export interface RelatorioOtimizacao {
  before: RamSnapshot;
  after: RamSnapshot;
  freed_disk_bytes: number;
  actions: string[];
  failures: string[];
}

export interface ModeloCatalogo {
  tag: string;
  family: string;
  label: string;
  params_b: number;
  active_params_b: number;
  moe: boolean;
  weights_bytes: number;
  context_k: number;
  vision: boolean;
  strength: number;
  tier: Nivel;
  focus: boolean;
  notes: string;
  footprint_bytes: number;
  /** Tokens por segundo estimados nesta máquina. Estimativa, não promessa. */
  estimated_tps: number;
  /** Cabe inteiro no acelerador, sem cair para a CPU no meio. */
  accelerated: boolean;
  supported: boolean;
  installed: boolean;
  fits_now: boolean;
  reason: string;
}

export interface Vaga {
  cargo: string;
  cargo_label: string;
  nivel: Nivel;
  modelo: string | null;
  modelo_label: string | null;
  footprint_bytes: number;
  estimated_tps: number;
  instalado: boolean;
  moe: boolean;
  aviso: string | null;
}

export type TipoReferencia = "propria" | "marca";

export interface Referencia {
  id: string;
  nome: string;
  caminho: string;
  bytes: number;
  tipo: TipoReferencia;
  nota: string;
}

export interface DesignSystem {
  cores: string;
  tipografia: string;
  tom_visual: string;
  evitar: string;
}

export type Provedor = "ollama" | "claude_code";

export interface StatusProvedor {
  claude_caminho: string | null;
  credencial_ignorada: string | null;
  provedor: Provedor;
  claude_disponivel: boolean;
  claude_versao: string | null;
}

export interface Skill {
  id: string;
  nome: string;
  texto: string;
  /** slug do cargo, ou vazio para todos. */
  cargo: string;
  ativa: boolean;
}

export interface Preferencias {
  provedor: Provedor;
  /** Liga a escolha manual de modelo por cargo. */
  avancado: boolean;
  /** slug do cargo -> tag do modelo. So vale com `avancado` ligado. */
  modelos: Record<string, string>;
  ds: DesignSystem;
  referencias: Referencia[];
  skills: Skill[];
}

export interface ResumoCofre {
  has_gemini_key: boolean;
  gemini_key_hint: string;
  saved_networks: string[];
  path: string;
}

export type Estagio =
  | "medindo_memoria"
  | "escolhendo_modelo"
  | "baixando_modelo"
  | "pensando"
  | "descarregando"
  | "concluido"
  | "falhou";

export interface EventoEstagio {
  step: number;
  role: string;
  network: string | null;
  stage: Estagio;
  model: string | null;
  detail: string;
  available_ram_bytes: number;
  percent: number | null;
  /** A mensagem que atravessou, no evento de conclusão. */
  handoff: string | null;
}

export interface ImagemGerada {
  path: string;
  bytes: number;
  model: string;
  aspect_ratio: string;
}

export interface Peca {
  rede: string;
  conceito: string;
  prompt_imagem: string;
  legenda: string;
  hashtags: string[];
  chamada_para_acao: string;
  imagem: ImagemGerada | null;
  publicado: boolean;
  motion_pedido?: boolean;
  roteiro_motion?: string | null;
  detalhe_publicacao: string;
  screenshot: string | null;
}

export interface RelatorioCampanha {
  run_id: string;
  run_dir: string;
  index_path: string;
  pecas: Peca[];
  rodadas: number;
  aprovado: boolean;
  parecer_auditor: string;
  avisos: string[];
}

export interface Credencial {
  username: string;
  password: string;
}

export interface PedidoCampanha {
  objetivo: string;
  redes: string[];
  credenciais: Record<string, Credencial>;
  salvar_credenciais: boolean;
  qualidade_imagem: "rapida" | "alta";
  simular: boolean;
  max_rodadas: number;
  pensamento_estendido: boolean;
  /** Idioma da entrega dos agentes: legenda, briefing e parecer. */
  idioma: string;
}

export interface PecaSalva {
  rede: string;
  conceito: string;
  prompt_imagem: string;
  legenda: string;
  hashtags: string[];
  chamada_para_acao: string;
  imagem: { path: string; aspect_ratio: string } | null;
  publicado: boolean;
  detalhe_publicacao: string;
  screenshot: string | null;
  motion_pedido?: boolean;
  roteiro_motion?: string | null;
}

export interface ResultadoCampanha {
  id: string;
  objetivo: string;
  redes: string[];
  aprovado: boolean;
  rodadas: number;
  simulado: boolean;
  parecer_auditor: string;
  avisos: string[];
  pecas: PecaSalva[];
  encerrada_em: string;
}

export interface CampanhaSalva {
  id: string;
  dir: string;
  index: string;
  turns: number;
  objetivo: string;
  redes: string[];
  pecas: number;
  publicadas: number;
  simulado: boolean;
  aprovado: boolean;
}

// -------------------------------------------------------------------- cerebro

export interface NoCerebro {
  type: string;
  context: string;
  created_at: number;
  updated_at: number;
  hits: number;
}

export interface ArestaCerebro {
  from: string;
  to: string;
  type: string;
  weight: number;
  uses: number;
  last_used: number;
}

export interface GrafoCerebro {
  nodes: Record<string, NoCerebro>;
  edges: ArestaCerebro[];
  schema_version: number;
  updated_at: number;
}

export interface VizinhoCerebro {
  node: string;
  type: string;
  weight: number;
  context: string;
}

export interface VistaNo {
  node: string;
  type: string;
  context: string;
  neighbors: VizinhoCerebro[];
}

export interface EstatisticasCerebro {
  nodes: number;
  edges: number;
  raw_bytes: number;
  compressed_bytes: number;
  ratio: number;
  path: string;
  updated_at: number;
}

// ------------------------------------------------------------- auditoria

export type OrigemMetrica = "manual" | "raspagem";
export type VereditoRede = "sem_base" | "divergir" | "seguir";
export type BaseRanking = "taxa" | "volume";

export interface RegistroMetrica {
  id: string;
  run_id: string;
  rede: string;
  publicado_em: string;
  url: string;
  conceito: string;
  impressoes: number;
  curtidas: number;
  comentarios: number;
  compartilhamentos: number;
  salvamentos: number;
  cliques: number;
  origem: OrigemMetrica;
  coletado_em: string;
}

export interface ItemRanking {
  id: string;
  conceito: string;
  publicado_em: string;
  taxa: number;
  multiplo: number;
  interacoes: number;
  impressoes: number;
}

export interface LeituraDaRede {
  rede: string;
  publicacoes: number;
  veredito: VereditoRede;
  base: BaseRanking;
  fora_da_base: number;
  mediana: number;
  multiplo_da_melhor: number;
  melhor_conceito: string;
  pior_conceito: string;
  ranking: ItemRanking[];
}

/** Um serviço de geração de arte, do jeito que a tela desenha. */
export interface CartaoImagem {
  slug: string;
  label: string;
  verificado: boolean;
  precisa_de_par: boolean;
  url_da_chave: string;
  tem_chave: boolean;
  dica: string;
  ativo: boolean;
}

/** Um cargo e o modelo Claude que o assume. */
export interface VagaClaude {
  cargo: string;
  nivel: Nivel;
  modelo: string;
  rotulo: string;
  porque: string;
}

/** O que o modal de movimento recebe quando a campanha para para perguntar. */
export interface PedidoMotion {
  rede: string;
  motivo: string;
}

/** Estado inicial das preferencias, antes de o Rust responder. */
export const PREFS_VAZIAS: Preferencias = {
  provedor: "ollama",
  avancado: false,
  modelos: {},
  ds: { cores: "", tipografia: "", tom_visual: "", evitar: "" },
  referencias: [],
  skills: [],
};

/** O navegador que publica — sonda da tela de preparação.
 *
 *  O Playwright chega pelo npm, mas o Chromium é um download à parte e preso
 *  à versão exata da biblioteca. Atualizar uma sem a outra deixa um estado que
 *  parece saudável até alguém clicar em publicar. */
export interface StatusNavegador {
  state: "pronto" | "ausente" | "semsidecar" | "semnode";
  caminho: string | null;
  detalhe: string;
}

export interface RelatorioNavegador {
  ok: boolean;
  passos: string[];
  erros: string[];
  status_final: StatusNavegador;
}

/** Uma falha que parou a campanha. */
export interface Falha {
  etapa: string;
  detalhe: string;
  pasta: string | null;
  sugestao: string | null;
}
