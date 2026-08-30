/** Os tipos que os componentes trazidos do app precisam.
 *
 *  Espelham src/types.ts, mas só o que a vitrine usa: importar o arquivo
 *  inteiro traria os tipos de Tauri, cofre e orquestrador junto, e nada disso
 *  existe numa página estática. */

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
  tier: "alto" | "medio" | "baixo";
  focus: boolean;
  notes: string;
  footprint_bytes: number;
  estimated_tps: number;
  accelerated: boolean;
  supported: boolean;
  installed: boolean;
  fits_now: boolean;
  reason: string;
}
