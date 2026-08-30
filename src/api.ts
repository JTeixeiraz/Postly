// Unica porta de entrada para o Rust. Nenhum componente chama `invoke` direto:
// se o nome de um comando mudar, muda em um lugar so.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  EstadoLocal,
  ProgressoLocal,
  CartaoModo,
  AvisoLimite,
  EsperaLimite,
  CampanhaSalva,
  Diagnostico,
  OllamaStatus,
  SondaSistema,
  EstatisticasCerebro,
  EventoEstagio,
  GrafoCerebro,
  ModeloCatalogo,
  PedidoCampanha,
  PlanoOtimizacao,
  ProgressoProvisao,
  Preferencias,
  Provedor,
  StatusProvedor,
  DesignSystem,
  TipoReferencia,
  PerfilComputacao,
  RamSnapshot,
  RelatorioCampanha,
  RelatorioOtimizacao,
  ResumoCofre,
  Vaga,
  VistaNo,
  RegistroMetrica,
  LeituraDaRede,
  VagaClaude,
  Atualizacao,
  Falha,
  ProgressoBaixa,
  PedidoMotion,
  RelatorioNavegador,
  StatusNavegador,
  ResultadoCampanha,
  CartaoImagem,
} from "./types";

export const api = {
  diagnostico: () => invoke<Diagnostico>("diagnostico"),
  memoria: () => invoke<RamSnapshot>("memoria"),
  computacao: () => invoke<PerfilComputacao>("computacao"),

  // Sondas da tela de preparo, rodadas em sequência.
  sondaSistema: () => invoke<SondaSistema>("sonda_sistema"),
  sondaMemoria: () => invoke<RamSnapshot>("sonda_memoria"),
  sondaAcelerador: () => invoke<PerfilComputacao>("sonda_acelerador"),
  verificarAtualizacao: () => invoke<Atualizacao>("verificar_atualizacao"),
  instalarAtualizacao: (url: string) => invoke<string>("instalar_atualizacao", { url }),
  sondaNavegador: () => invoke<StatusNavegador>("sonda_navegador"),
  provisionarNavegador: () => invoke<RelatorioNavegador>("provisionar_navegador"),
  sondaOllama: () => invoke<OllamaStatus>("sonda_ollama"),
  provisionarOllama: () => invoke<{ ok: boolean; steps: string[]; errors: string[] }>("provisionar_ollama"),

  planoOtimizacao: () => invoke<PlanoOtimizacao>("plano_otimizacao"),
  otimizar: (caminhos: string[], permitirElevacao: boolean) =>
    invoke<RelatorioOtimizacao>("otimizar", { caminhos, permitirElevacao }),

  elenco: () => invoke<Vaga[]>("elenco"),
  catalogoModelos: () => invoke<ModeloCatalogo[]>("catalogo_modelos"),
  baixarModelo: (tag: string) => invoke<void>("baixar_modelo", { tag }),
  removerModelo: (tag: string) => invoke<void>("remover_modelo", { tag }),
  modelosCarregados: () => invoke<{ name: string; bytes: number }[]>("modelos_carregados"),

  preferencias: () => invoke<Preferencias>("preferencias"),
  definirModoAvancado: (ligado: boolean) =>
    invoke<Preferencias>("definir_modo_avancado", { ligado }),
  definirModeloDoCargo: (cargo: string, tag: string) =>
    invoke<Preferencias>("definir_modelo_do_cargo", { cargo, tag }),

  salvarReferencia: (nome: string, dados: string, tipo: TipoReferencia, nota: string) =>
    invoke<Preferencias>("salvar_referencia", { nome, dados, tipo, nota }),
  removerReferencia: (id: string) => invoke<Preferencias>("remover_referencia", { id }),
  anotarReferencia: (id: string, nota: string) =>
    invoke<Preferencias>("anotar_referencia", { id, nota }),
  salvarDesignSystem: (ds: DesignSystem) => invoke<Preferencias>("salvar_design_system", { ds }),

  estadoImagemLocal: () => invoke<EstadoLocal>("estado_imagem_local"),
  baixarMotorLocal: () => invoke<string>("baixar_motor_local"),
  baixarModeloLocal: (id: string) => invoke<string>("baixar_modelo_local", { id }),
  removerModeloLocal: (id: string) => invoke<void>("remover_modelo_local", { id }),
  modosDeDesempenho: () => invoke<CartaoModo[]>("modos_de_desempenho"),
  definirModo: (slug: string) => invoke<void>("definir_modo", { slug }),
  statusProvedor: () => invoke<StatusProvedor>("status_provedor"),
  definirProvedor: (provedor: Provedor) =>
    invoke<Preferencias>("definir_provedor", { provedor }),

  salvarSkill: (id: string, nome: string, texto: string, cargo: string, ativa: boolean) =>
    invoke<Preferencias>("salvar_skill", { id, nome, texto, cargo, ativa }),
  removerSkill: (id: string) => invoke<Preferencias>("remover_skill", { id }),
  previaDeSkills: (cargo: string) => invoke<string>("previa_de_skills", { cargo }),
  descarregarModelos: () => invoke<string[]>("descarregar_modelos"),

  resumoCofre: () => invoke<ResumoCofre>("resumo_cofre"),
  salvarChaveGemini: (chave: string) => invoke<ResumoCofre>("salvar_chave_gemini", { chave }),
  validarChaveGemini: () => invoke<string>("validar_chave_gemini"),
  esquecerCredenciais: (rede: string) => invoke<ResumoCofre>("esquecer_credenciais", { rede }),

  iniciarCampanha: (pedido: PedidoCampanha) =>
    invoke<RelatorioCampanha>("iniciar_campanha", { pedido }),
  listarCampanhas: () => invoke<CampanhaSalva[]>("listar_campanhas"),
  lerMarkdown: (caminho: string) => invoke<string>("ler_markdown", { caminho }),
  abrirNoSistema: (caminho: string) => invoke<void>("abrir_no_sistema", { caminho }),
  fecharNavegador: () => invoke<void>("fechar_navegador"),

  cerebroStats: () => invoke<EstatisticasCerebro>("cerebro_stats"),
  cerebroGrafo: () => invoke<GrafoCerebro>("cerebro_grafo"),
  cerebroNode: (id: string, pesoMinimo: number, topK: number) =>
    invoke<VistaNo | null>("cerebro_node", { id, pesoMinimo, topK }),
  cerebroBuscar: (termos: string[], topK: number) =>
    invoke<VistaNo[]>("cerebro_buscar", { termos, topK }),
  cerebroEscreverNode: (id: string, tipo: string, contexto: string) =>
    invoke<EstatisticasCerebro>("cerebro_escrever_node", { id, tipo, contexto }),
  cerebroEscreverAresta: (origem: string, destino: string, tipo: string, peso: number) =>
    invoke<EstatisticasCerebro>("cerebro_escrever_aresta", { origem, destino, tipo, peso }),
  cerebroRemoverNode: (id: string) => invoke<EstatisticasCerebro>("cerebro_remover_node", { id }),
  cerebroDecair: () => invoke<number>("cerebro_decair"),

  // auditoria de desempenho
  listarMetricas: () => invoke<RegistroMetrica[]>("listar_metricas"),
  leituraDesempenho: () => invoke<LeituraDaRede[]>("leitura_desempenho"),
  registrarMetrica: (m: Partial<RegistroMetrica>) =>
    invoke<RegistroMetrica[]>("registrar_metrica", { m }),
  removerMetrica: (id: string) => invoke<RegistroMetrica[]>("remover_metrica", { id }),
  coletarMetricas: (rede: string, limite: number) =>
    invoke<RegistroMetrica[]>("coletar_metricas", { rede, limite }),
  analisarDesempenho: () => invoke<string>("analisar_desempenho"),

  pecasDaCampanha: (dir: string) =>
    invoke<ResultadoCampanha | null>("pecas_da_campanha", { dir }),

  provedoresDeImagem: () => invoke<CartaoImagem[]>("provedores_de_imagem"),
  definirProvedorImagem: (slug: string) =>
    invoke<CartaoImagem[]>("definir_provedor_imagem", { slug }),
  salvarChaveDeImagem: (slug: string, chave: string) =>
    invoke<CartaoImagem[]>("salvar_chave_de_imagem", { slug, chave }),
  testarProvedorImagem: (slug: string) => invoke<string>("testar_provedor_imagem", { slug }),

  elencoClaude: () => invoke<VagaClaude[]>("elenco_claude"),
  responderMotion: (aceitar: boolean) => invoke<void>("responder_motion", { aceitar }),
  responderLimite: (esperar: boolean) => invoke<void>("responder_limite", { esperar }),
};

/** A campanha parou e espera a decisao sobre animar a peca. */
/** A campanha parou. O Rust já disparou a notificação do sistema; aqui abre
 *  o modal com o detalhe. */
/** Progresso do download da atualização. */
export function ouvirAtualizacao(cb: (e: ProgressoBaixa) => void): Promise<UnlistenFn> {
  return listen<ProgressoBaixa>("postly://atualizacao", (evento) => cb(evento.payload));
}

export function ouvirFalha(cb: (e: Falha) => void): Promise<UnlistenFn> {
  return listen<Falha>("postly://falha", (evento) => cb(evento.payload));
}

export function ouvirMotion(cb: (e: PedidoMotion) => void): Promise<UnlistenFn> {
  return listen<PedidoMotion>("postly://motion", (evento) => cb(evento.payload));
}

/** O aviso de cota esgotada, e os dois eventos da espera.
 *
 *  `ouvirEsperaLimite` e `ouvirFimDoLimite` existem porque a espera pode durar
 *  horas: sem eles a tela ficaria mostrando o modal de decisão o tempo todo,
 *  como se ninguém tivesse decidido nada. */
export function ouvirImagemLocal(cb: (e: ProgressoLocal) => void): Promise<UnlistenFn> {
  return listen<ProgressoLocal>("postly://imagem-local", (evento) => cb(evento.payload));
}

export function ouvirLimite(cb: (e: AvisoLimite) => void): Promise<UnlistenFn> {
  return listen<AvisoLimite>("postly://limite", (evento) => cb(evento.payload));
}

export function ouvirEsperaLimite(cb: (e: EsperaLimite) => void): Promise<UnlistenFn> {
  return listen<EsperaLimite>("postly://limite-esperando", (evento) => cb(evento.payload));
}

export function ouvirFimDoLimite(cb: () => void): Promise<UnlistenFn> {
  return listen("postly://limite-fim", () => cb());
}

export function ouvirEstagios(cb: (e: EventoEstagio) => void): Promise<UnlistenFn> {
  return listen<EventoEstagio>("postly://estagio", (evento) => cb(evento.payload));
}

/** Andamento da instalacao do Ollama, linha a linha do instalador. */
export function ouvirProvisao(cb: (e: ProgressoProvisao) => void): Promise<UnlistenFn> {
  return listen<ProgressoProvisao>("postly://provisao", (evento) => cb(evento.payload));
}

export function ouvirDownloads(
  cb: (e: { model: string; status: string; percent: number }) => void
): Promise<UnlistenFn> {
  return listen<{ model: string; status: string; percent: number }>(
    "postly://download",
    (evento) => cb(evento.payload)
  );
}
