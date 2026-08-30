import { formatarBytes, formatarNumero, useIdioma } from "../i18n";
import type { ModeloCatalogo } from "../types";
import MarcaModelo from "./MarcaModelo";
import { IconCheck, IconDownload, IconSpinner, IconTrash } from "./Icons";

interface Props {
  m: ModeloCatalogo;
  /** 0 a 100 enquanto baixa; ausente quando parado. */
  progresso?: number;
  removendo?: boolean;
  onBaixar: (tag: string) => void;
  onRemover: (tag: string) => void;
}

/** Uma linha do catalogo, com a acao que faz sentido para o estado dela.
 *
 *  A tela antiga so informava. Poder baixar e remover daqui e o que transforma
 *  a lista em ferramenta: a decisao de qual modelo ter no disco e do usuario, e
 *  ate agora ela so acontecia por acidente, quando um cargo precisava. */
export default function LinhaModelo({ m, progresso, removendo, onBaixar, onRemover }: Props) {
  const { d, f, idioma } = useIdioma();
  const baixando = progresso !== undefined;

  return (
    <div className="modelo" data-fora={!m.supported}>
      <div className="modelo__esq">
        <MarcaModelo familia={m.family} />
        <div>
          <div className="modelo__id">
            <span className="modelo__nome">{m.label}</span>
            <span className="modelo__tag">{m.tag}</span>
            {m.installed && (
              <span className="tag" data-tone="ok">
                <IconCheck size={11} />
                {d.common.installed}
              </span>
            )}
            {!m.installed && m.supported && !m.fits_now && (
              <span className="tag" data-tone="warn">
                <span className="tag__dot" />
                {d.models.notNow}
              </span>
            )}
            {m.vision && <span className="tag">{d.models.vision}</span>}
          </div>
          <div className="modelo__nota">{m.notes}</div>
        </div>
      </div>

      <div className="modelo__dir">
        <div className="modelo__num">
          <span className="modelo__peso">{formatarBytes(m.footprint_bytes, idioma)}</span>
          <span className="modelo__vel" data-rapido={m.estimated_tps >= 4}>
            ≈ {formatarNumero(m.estimated_tps, idioma)} tok/s
          </span>
          <span className="modelo__vel">
            {m.moe
              ? f(d.models.moeNote, {
                  active: formatarNumero(m.active_params_b, idioma),
                  total: formatarNumero(m.params_b, idioma, 0),
                })
              : f(d.models.denseNote, { total: formatarNumero(m.params_b, idioma, 0) })}
          </span>
        </div>

        {/* A acao so existe onde ela faz sentido: modelo que esta fora do
            alcance da maquina nao ganha botao de baixar. */}
        <div className="modelo__acao">
          {m.installed ? (
            <button
              className="btn btn--quiet btn--sm"
              onClick={() => onRemover(m.tag)}
              disabled={removendo}
              title={d.models.remove}
            >
              {removendo ? <IconSpinner size={13} /> : <IconTrash size={13} />}
              {removendo ? d.models.removing : d.models.remove}
            </button>
          ) : m.supported ? (
            <button className="btn btn--sm" onClick={() => onBaixar(m.tag)} disabled={baixando}>
              {baixando ? <IconSpinner size={13} /> : <IconDownload size={13} />}
              {baixando ? `${progresso.toFixed(0)}%` : d.models.download}
            </button>
          ) : null}
        </div>
      </div>

      {baixando && (
        <div className="modelo__barra">
          <span style={{ transform: `scaleX(${Math.max(progresso, 1) / 100})` }} />
        </div>
      )}
    </div>
  );
}
