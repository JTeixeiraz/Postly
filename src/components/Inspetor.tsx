import { useIdioma } from "../i18n";
import type { CenaVideo, NotaDeCena, RoteiroVideo } from "../types";

/** O inspetor: o que o Motion Designer decidiu nesta cena, e por quê.
 *
 *  TODOS OS CAMPOS SÃO SÓ LEITURA, menos a nota. Isso é o produto, não uma
 *  limitação: se a pessoa pudesse trocar o movimento num seletor, ela estaria
 *  editando o vídeo, e editar vídeo é o trabalho que este programa existe para
 *  fazer no lugar dela.
 *
 *  Mostrar as decisões mesmo sem deixar mexer tem uma razão: sem elas a pessoa
 *  não tem vocabulário para apontar. "Está estranho" não corrige nada; "a
 *  câmera aproxima demais e o texto cobre o rosto" corrige — e ela só escreve
 *  a segunda se tiver visto que existe um campo chamado movimento e outro
 *  chamado pouso. */
export default function Inspetor({
  roteiro,
  indice,
  segundo,
  notas,
  aoAnotar,
}: {
  roteiro: RoteiroVideo;
  /** Base 1, como a linha do tempo numera. */
  indice: number | null;
  segundo: number;
  notas: NotaDeCena[];
  aoAnotar: (n: NotaDeCena[]) => void;
}) {
  const { d, f, idioma } = useIdioma();

  if (indice === null) {
    return (
      <aside className="inspetor">
        <p className="hint">{d.inspetor.pick}</p>
        <Look roteiro={roteiro} />
      </aside>
    );
  }

  const cena: CenaVideo | undefined = roteiro.cenas[indice - 1];
  if (!cena) return null;

  const atual = notas.find((n) => n.cena === indice);

  const escrever = (texto: string) => {
    const resto = notas.filter((n) => n.cena !== indice);
    // Nota vazia é nota apagada: guardar uma string em branco faria o bloco de
    // correção chegar no modelo com um item que não diz nada, e ele tentaria
    // atender.
    aoAnotar(
      texto.trim()
        ? [...resto, { cena: indice, segundo, texto }].sort((a, b) => a.cena - b.cena)
        : resto
    );
  };

  return (
    <aside className="inspetor">
      <div className="inspetor__topo">
        <span className="tag">{f(d.inspetor.scene, { n: indice })}</span>
        <span className="hint num">{segundos(cena.dur_s, idioma)}</span>
      </div>

      <h3 className="inspetor__titulo">{cena.titulo || d.inspetor.noTitle}</h3>
      {cena.subtitulo && <p className="hint">{cena.subtitulo}</p>}

      <dl className="inspetor__campos">
        <Campo k={d.inspetor.type} v={d.tipoCena[cena.tipo]} />
        <Campo k={d.inspetor.movement} v={d.direcao[cena.direcao.movimento]} />
        <Campo k={d.inspetor.focus} v={d.direcao[cena.direcao.foco]} />
        <Campo k={d.inspetor.landing} v={d.direcao[cena.direcao.pouso]} />
        <Campo k={d.inspetor.entrance} v={d.direcao[cena.direcao.entrada]} />
        {cena.imagens.length > 0 && (
          <Campo k={d.inspetor.images} v={cena.imagens.join(", ")} />
        )}
        {cena.narracao && <Campo k={d.inspetor.voice} v={cena.narracao} />}
      </dl>

      <label className="field">
        <span>{d.inspetor.noteLabel}</span>
        <textarea
          rows={4}
          value={atual?.texto ?? ""}
          placeholder={d.inspetor.notePlaceholder}
          onChange={(e) => escrever(e.target.value)}
        />
        {/* O instante fica gravado junto da nota. "Aos 4,2s o texto cobre o
            rosto" é uma instrução que o cargo consegue seguir; "está errado"
            não é. */}
        <span className="field__help">
          {f(d.inspetor.noteAt, { s: segundos(segundo, idioma) })}
        </span>
      </label>

      <Look roteiro={roteiro} />
    </aside>
  );
}

/** Segundos no idioma da tela.
 *
 *  `toFixed` sempre escreve ponto, e o resto do app escreve "16,5 GB" em
 *  português. Um número com ponto no meio de uma tela com vírgula lê como texto
 *  importado de outro lugar. */
function segundos(s: number, idioma: string) {
  return `${s.toLocaleString(idioma === "pt" ? "pt-BR" : "en-US", {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  })}s`;
}

function Campo({ k, v }: { k: string; v: string }) {
  return (
    <>
      <dt>{k}</dt>
      <dd>{v}</dd>
    </>
  );
}

/** A direção do vídeo inteiro.
 *
 *  Fica no rodapé do inspetor e não numa aba própria: é uma decisão só, tomada
 *  uma vez, e uma aba para três campos seria navegação a mais do que conteúdo. */
function Look({ roteiro }: { roteiro: RoteiroVideo }) {
  const { d, f } = useIdioma();
  return (
    <div className="inspetor__look">
      <span className="read__k">{d.inspetor.look}</span>
      <p className="hint">
        {f(d.inspetor.energy, { n: Math.round(roteiro.look.energia * 100) })}
        {roteiro.look.vinheta && ` · ${d.inspetor.vignette}`}
        {roteiro.look.filete && ` · ${d.inspetor.rule}`}
      </p>
      {roteiro.racional && <p className="hint">{roteiro.racional}</p>}
    </div>
  );
}
