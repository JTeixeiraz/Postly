import { formatarBytes, formatarNumero, useIdioma } from "../i18n";
import type { Idioma } from "../i18n";
import type { ModeloCatalogo, Preferencias } from "../types";
import MarcaModelo from "./MarcaModelo";
import Selecao from "./Selecao";

/** Os quatro cargos, na ordem em que o despacho passa por eles. */
const CARGOS = [
  { slug: "diretor-geral", chave: "diretor_geral" },
  { slug: "gerente-setor", chave: "gerente_setor" },
  { slug: "criador", chave: "criador" },
  { slug: "auditor", chave: "auditor" },
] as const;

interface Props {
  modelos: ModeloCatalogo[];
  prefs: Preferencias;
  onEscolher: (cargo: string, tag: string) => void;
}

function descrever(m: ModeloCatalogo, idioma: Idioma): string {
  return [
    formatarBytes(m.footprint_bytes, idioma),
    `${formatarNumero(m.estimated_tps, idioma)} tok/s`,
  ].join(" · ");
}

/** Escolha manual de modelo por cargo.
 *
 *  Fica atras de um interruptor porque a escolha automatica e a certa para
 *  quase todo mundo: ela remede a memoria a cada troca. Quem liga isto assume
 *  que o modelo fixado pode nao caber depois, e o app avisa em vez de trocar
 *  pelas costas. */
export default function CargosAvancado({ modelos, prefs, onEscolher }: Props) {
  const { d, f, idioma } = useIdioma();
  const disponiveis = modelos.filter((m) => m.supported);

  return (
    <div className="cargos">
      {CARGOS.map(({ slug, chave }) => {
        const escolhido = prefs.modelos[slug] ?? "";
        const m = disponiveis.find((x) => x.tag === escolhido);
        return (
          <label className="cargo" key={slug}>
            <span className="cargo__nome">{f(d.models.roleModel, { cargo: d.roles[chave] })}</span>
            <Selecao
              valor={escolhido}
              onEscolher={(v) => onEscolher(slug, v)}
              antes={m ? <MarcaModelo familia={m.family} size={15} /> : undefined}
              opcoes={[
                { valor: "", rotulo: d.models.auto, nota: d.models.autoWhy },
                ...disponiveis.map((x) => ({
                  valor: x.tag,
                  rotulo: x.label,
                  nota: descrever(x, idioma),
                })),
              ]}
            />
            <span className="cargo__nota">
              {!m ? (
                d.models.autoWhy
              ) : !m.installed ? (
                d.models.willDownload
              ) : !m.fits_now ? (
                <em className="aviso">{d.models.tooBigNow}</em>
              ) : (
                `${m.tag}`
              )}
            </span>
          </label>
        );
      })}
    </div>
  );
}
