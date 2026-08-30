import { useRef, useState } from "react";
import { api } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { DesignSystem, Preferencias, Referencia, TipoReferencia } from "../types";
import { IconTrash } from "./Icons";

interface Props {
  prefs: Preferencias;
  onMudar: (p: Preferencias) => void;
}

/** Referencias visuais e identidade da marca.
 *
 *  Fica recolhido porque e opcional: quem esta rodando a primeira campanha nao
 *  precisa disto para chegar ao resultado, e mostrar tudo aberto so aumentaria
 *  a tela que a pessoa precisa entender antes de apertar o botao.
 *
 *  As duas listas sao separadas de proposito. Material da propria marca vai
 *  como imagem para o modelo e pode aparecer na peca; referencia de estilo vai
 *  so como texto, porque mandar a arte de outra marca para o modelo copiar e o
 *  caminho mais curto para a peca sair com logotipo alheio. */
export default function Referencias({ prefs, onMudar }: Props) {
  const { d, idioma } = useIdioma();
  const [aberto, setAberto] = useState(false);
  const [erro, setErro] = useState<string | null>(null);
  const [recado, setRecado] = useState<string | null>(null);
  const [ds, setDs] = useState<DesignSystem>(prefs.ds);
  const entradaPropria = useRef<HTMLInputElement>(null);
  const entradaMarca = useRef<HTMLInputElement>(null);

  const enviar = async (arquivo: File | undefined, tipo: TipoReferencia) => {
    if (!arquivo) return;
    setErro(null);
    try {
      const dados = await new Promise<string>((ok, falha) => {
        const leitor = new FileReader();
        leitor.onload = () => ok(String(leitor.result));
        leitor.onerror = () => falha(leitor.error);
        leitor.readAsDataURL(arquivo);
      });
      onMudar(await api.salvarReferencia(arquivo.name, dados, tipo, ""));
    } catch (e) {
      setErro(String(e));
    }
  };

  const lista = (tipo: TipoReferencia) => prefs.referencias.filter((r) => r.tipo === tipo);

  const salvarDs = async () => {
    setErro(null);
    try {
      onMudar(await api.salvarDesignSystem(ds));
      setRecado(d.refs.saved);
    } catch (e) {
      setErro(String(e));
    }
  };

  const quantas = prefs.referencias.length;

  return (
    <section className="card">
      <button className="card__topo dobra" onClick={() => setAberto((a) => !a)} aria-expanded={aberto}>
        <span className="dobra__seta" data-aberto={aberto} aria-hidden />
        <h2>{d.refs.title}</h2>
        <span className="hint push">
          {quantas > 0 ? `${quantas} · ${d.refs.lead}` : d.refs.lead}
        </span>
      </button>

      {aberto && (
        <>
          {erro && (
            <div className="note" data-tone="alert">
              <span>{erro}</span>
            </div>
          )}

          <div className="refs-par">
            {(
              [
                ["propria", d.refs.own, d.refs.ownWhy, entradaPropria] as const,
                ["marca", d.refs.brand, d.refs.brandWhy, entradaMarca] as const,
              ]
            ).map(([tipo, titulo, porque, ref]) => (
              <div className="refs-col" key={tipo}>
                <div className="stack stack--tight">
                  <span className="card__titulo">{titulo}</span>
                  <span className="hint">{porque}</span>
                </div>

                <div className="refs-lista">
                  {lista(tipo).length === 0 ? (
                    <p className="hint">{d.refs.empty}</p>
                  ) : (
                    lista(tipo).map((r) => (
                      <Miniatura key={r.id} r={r} idioma={idioma} onMudar={onMudar} />
                    ))
                  )}
                </div>

                <div className="row row--tight">
                  <input
                    ref={ref}
                    type="file"
                    accept="image/png,image/jpeg,image/webp"
                    hidden
                    onChange={(e) => {
                      void enviar(e.target.files?.[0], tipo);
                      e.target.value = "";
                    }}
                  />
                  <button className="btn btn--sm" onClick={() => ref.current?.click()}>
                    {d.refs.add}
                  </button>
                  <span className="hint">{d.refs.formats}</span>
                </div>
              </div>
            ))}
          </div>

          <div className="note">
            <strong>{d.refs.exampleTitle}</strong>
            <span>{d.refs.exampleBody}</span>
          </div>

          {/* ── identidade visual ─────────────────────────────────── */}
          <div className="stack stack--tight">
            <span className="card__titulo">{d.refs.ds}</span>
            <span className="hint">{d.refs.dsWhy}</span>
          </div>

          <div className="auto-grid">
            {(
              [
                ["cores", d.refs.colors, d.refs.colorsPlaceholder],
                ["tipografia", d.refs.type, d.refs.typePlaceholder],
                ["tom_visual", d.refs.mood, d.refs.moodPlaceholder],
                ["evitar", d.refs.avoid, d.refs.avoidPlaceholder],
              ] as const
            ).map(([campo, rotulo, exemplo]) => (
              <label className="field" key={campo}>
                <span>{rotulo}</span>
                <input
                  type="text"
                  value={ds[campo]}
                  placeholder={exemplo}
                  onChange={(e) => setDs((v) => ({ ...v, [campo]: e.target.value }))}
                  onBlur={salvarDs}
                />
              </label>
            ))}
          </div>

          {recado && <span className="hint">{recado}</span>}
        </>
      )}
    </section>
  );
}

function Miniatura({
  r,
  idioma,
  onMudar,
}: {
  r: Referencia;
  idioma: "pt" | "en";
  onMudar: (p: Preferencias) => void;
}) {
  const { d } = useIdioma();
  return (
    <div className="ref">
      <div className="ref__topo">
        <span className="ref__nome" title={r.nome}>
          {r.nome}
        </span>
        <button
          className="btn btn--quiet btn--sm"
          onClick={() => api.removerReferencia(r.id).then(onMudar)}
          title={d.models.remove}
        >
          <IconTrash size={13} />
        </button>
      </div>
      <span className="hint num">{formatarBytes(r.bytes, idioma)}</span>
      <input
        type="text"
        placeholder={d.refs.notePlaceholder}
        defaultValue={r.nota}
        aria-label={d.refs.noteFor}
        onBlur={(e) => {
          // A nota e o que diz ao modelo o que olhar na imagem. Sem ela, uma
          // referencia de enquadramento vira referencia de assunto.
          if (e.target.value !== r.nota) {
            void api.anotarReferencia(r.id, e.target.value).then(onMudar).catch(() => {});
          }
        }}
      />
    </div>
  );
}
