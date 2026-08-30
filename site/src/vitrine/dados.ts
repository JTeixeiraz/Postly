import type { GrafoCerebro, ModeloCatalogo } from "./tipos";

/** Dados de exemplo para a vitrine.
 *
 *  São reais no formato e plausíveis no conteúdo: os modelos existem na
 *  biblioteca do Ollama, os tok/s são da ordem medida numa máquina sem GPU, e
 *  o grafo tem a forma que um cérebro real ganha depois de algumas campanhas.
 *  Inventar números redondos aqui faria a vitrine mentir sobre o produto. */

const modelo = (m: Partial<ModeloCatalogo> & { tag: string; family: string; label: string }): ModeloCatalogo => ({
  params_b: 14, active_params_b: 14, moe: false, weights_bytes: 9_300_000_000,
  context_k: 128, vision: false, strength: 70, tier: "alto", focus: true,
  notes: "", footprint_bytes: 9_300_000_000, estimated_tps: 1.2,
  accelerated: false, supported: true, installed: false, fits_now: true, reason: "",
  ...m,
});

export const MODELOS: ModeloCatalogo[] = [
  modelo({
    tag: "qwen3:30b-a3b", family: "Qwen", label: "Qwen 3 30B-A3B (MoE)",
    params_b: 30, active_params_b: 3, moe: true,
    weights_bytes: 19_000_000_000, footprint_bytes: 20_100_000_000,
    estimated_tps: 5.7, installed: true,
    notes: "Só os especialistas ativos passam pela CPU — por isso é o mais rápido aqui.",
  }),
  modelo({
    tag: "gpt-oss:20b", family: "OpenAI", label: "GPT-OSS 20B (MoE)",
    params_b: 20, active_params_b: 3.6, moe: true,
    weights_bytes: 14_000_000_000, footprint_bytes: 15_400_000_000,
    estimated_tps: 4.9,
    notes: "Raciocínio forte com pegada modesta. Bom para o cargo que decide.",
  }),
  modelo({
    tag: "llama3.3:70b", family: "Meta", label: "Llama 3.3 70B",
    params_b: 70, active_params_b: 70,
    weights_bytes: 43_000_000_000, footprint_bytes: 45_200_000_000,
    estimated_tps: 0.3, supported: false, fits_now: false,
    reason: "Não cabe: precisa de 45,2 GB e há 21,5 GB livres.",
    notes: "Denso e caro. Só faz sentido com muita VRAM.",
  }),
  modelo({
    tag: "gemma3:4b", family: "Google", label: "Gemma 3 4B",
    params_b: 4, active_params_b: 4, tier: "baixo",
    weights_bytes: 3_300_000_000, footprint_bytes: 3_900_000_000,
    estimated_tps: 12.4, installed: true, vision: true,
    notes: "Cumpre briefing pronto sem inventar. Enxerga imagem.",
  }),
  modelo({
    tag: "deepseek-r1:14b", family: "DeepSeek", label: "DeepSeek R1 14B",
    params_b: 14, active_params_b: 14, tier: "medio",
    weights_bytes: 9_000_000_000, footprint_bytes: 9_800_000_000,
    estimated_tps: 1.1,
    notes: "Pensa antes de responder. Serve bem ao auditor.",
  }),
  modelo({
    tag: "mistral-small:24b", family: "Mistral", label: "Mistral Small 24B",
    params_b: 24, active_params_b: 24, tier: "medio",
    weights_bytes: 14_000_000_000, footprint_bytes: 15_100_000_000,
    estimated_tps: 0.7, fits_now: false,
    notes: "Cabe no disco, mas não na memória livre agora.",
  }),
  modelo({
    tag: "phi-4:14b", family: "Microsoft", label: "Phi-4 14B",
    params_b: 14, active_params_b: 14, tier: "medio",
    weights_bytes: 9_100_000_000, footprint_bytes: 9_900_000_000,
    estimated_tps: 1.2,
    notes: "Pequeno e obediente. Bom custo por token.",
  }),
  modelo({
    tag: "granite3.3:8b", family: "IBM", label: "Granite 3.3 8B",
    params_b: 8, active_params_b: 8, tier: "baixo",
    weights_bytes: 4_900_000_000, footprint_bytes: 5_500_000_000,
    estimated_tps: 3.8,
    notes: "Licença permissiva e saída previsível.",
  }),
];

/** Os quatro cargos, na ordem em que o despacho passa. */
export const POSTAS = [
  { cargo: "Diretor Geral", modelo: "qwen3:30b-a3b", nivel: "alto", nota: "decide a linha" },
  { cargo: "Gerente de Setor", modelo: "qwen3:30b-a3b", nivel: "alto", nota: "Instagram" },
  { cargo: "Criador de Conteúdo", modelo: "gemma3:4b", nivel: "baixo", nota: "executa o briefing" },
  { cargo: "Auditor", modelo: "deepseek-r1:14b", nivel: "médio", nota: "julga a peça" },
];

/** Um cérebro depois de algumas campanhas: os pesos não são redondos porque
 *  nasceram de reforço e decaimento, não de alguém digitando. */
export const GRAFO: GrafoCerebro = {
  schema_version: 1,
  updated_at: Date.now(),
  nodes: Object.fromEntries(
    [
      "publico_alvo", "tom_de_voz", "instagram", "linkedin", "produto",
      "concorrente", "prova_social", "objecao_preco", "sazonalidade",
      "formato_carrossel", "identidade_visual", "campanha_agosto",
    ].map((id) => [id, { type: "conceito", context: "", created_at: 0, updated_at: 0, hits: 3 }])
  ),
  edges: [
    { from: "publico_alvo", to: "tom_de_voz", type: "define", weight: 0.94, uses: 12, last_used: 0 },
    { from: "publico_alvo", to: "instagram", type: "vive_em", weight: 0.81, uses: 9, last_used: 0 },
    { from: "publico_alvo", to: "objecao_preco", type: "levanta", weight: 0.63, uses: 5, last_used: 0 },
    { from: "tom_de_voz", to: "identidade_visual", type: "combina", weight: 0.77, uses: 7, last_used: 0 },
    { from: "produto", to: "prova_social", type: "sustenta", weight: 0.88, uses: 11, last_used: 0 },
    { from: "produto", to: "concorrente", type: "compete", weight: 0.52, uses: 4, last_used: 0 },
    { from: "instagram", to: "formato_carrossel", type: "prefere", weight: 0.71, uses: 6, last_used: 0 },
    { from: "linkedin", to: "prova_social", type: "prefere", weight: 0.68, uses: 5, last_used: 0 },
    { from: "campanha_agosto", to: "instagram", type: "rodou_em", weight: 0.59, uses: 3, last_used: 0 },
    { from: "campanha_agosto", to: "sazonalidade", type: "considera", weight: 0.41, uses: 2, last_used: 0 },
    { from: "objecao_preco", to: "prova_social", type: "responde", weight: 0.74, uses: 6, last_used: 0 },
    { from: "identidade_visual", to: "formato_carrossel", type: "restringe", weight: 0.35, uses: 2, last_used: 0 },
    { from: "concorrente", to: "sazonalidade", type: "explora", weight: 0.29, uses: 1, last_used: 0 },
    { from: "tom_de_voz", to: "linkedin", type: "ajusta", weight: 0.46, uses: 3, last_used: 0 },
  ],
};
