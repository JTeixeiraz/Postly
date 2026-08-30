import { useEffect, useRef, useState } from "react";
import { motion, useScroll, useSpring, useTransform } from "motion/react";
import Trilha from "./componentes/Trilha";
import Comando from "./componentes/Comando";
import Apresentacao from "./componentes/Apresentacao";
import Vitrine from "./componentes/Vitrine";
import Lingua from "./componentes/Lingua";
import MarcaClaude from "./componentes/MarcaClaude";
import Trama from "./bits/Trama";
import TextLoop from "./bits/TextLoop";
import BorderGlow from "./bits/BorderGlow";
import { useIdioma } from "./i18n";

const REPO = "https://github.com/JTeixeiraz/Postly";

/** Os modelos de cada cargo não são texto: são etiquetas do Ollama, e uma
 *  etiqueta traduzida deixaria de ser copiável para o terminal. */
const MODELOS_DOS_CARGOS = ["qwen3:30b-a3b", "qwen3:30b-a3b", "gemma3:4b", "qwen3:14b"];

/** Marca uma seção como visitada quando ela entra na tela.
 *
 *  A posta na espinha acende a partir disto. Usa IntersectionObserver e não
 *  cálculo de scroll: o observer não roda a cada quadro e não força layout. */
function usarVisivel<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [visivel, setVisivel] = useState(false);

  useEffect(() => {
    const alvo = ref.current;
    if (!alvo) return;
    const obs = new IntersectionObserver(
      ([e]) => e.isIntersecting && setVisivel(true),
      { rootMargin: "-18% 0px -55% 0px" }
    );
    obs.observe(alvo);
    return () => obs.disconnect();
  }, []);

  return { ref, visivel };
}

function Secao({
  children,
  estreita = false,
  id,
}: {
  children: React.ReactNode;
  estreita?: boolean;
  id?: string;
}) {
  const { ref, visivel } = usarVisivel<HTMLElement>();
  return (
    <section ref={ref} id={id} className={`secao${estreita ? " secao--estreita" : ""}`}>
      <span className="espinha__posta" data-passou={visivel} aria-hidden />
      <div className="secao__interno">{children}</div>
    </section>
  );
}

export default function App() {
  const { d } = useIdioma();
  const { scrollYProgress } = useScroll();
  // Mola em vez do valor cru: o scroll de trackpad chega em saltos, e a linha
  // seguindo o valor direto treme.
  const avanco = useSpring(scrollYProgress, { stiffness: 90, damping: 26, restDelta: 0.001 });
  const escala = useTransform(avanco, [0, 1], [0, 1]);

  const cargos = d.cargos.map((c, i) => ({ ...c, modelo: MODELOS_DOS_CARGOS[i] }));

  return (
    <div className="pagina">
      <div className="espinha" aria-hidden>
        <div className="espinha__fio" />
        <motion.div className="espinha__aceso" style={{ scaleY: escala }} />
      </div>

      <header className="cabecalho">
        <a className="cabecalho__marca" href="#inicio">
          <Marca />
          <span>postly</span>
        </a>
        <nav className="cabecalho__nav">
          <a href="#como">{d.nav.como}</a>
          <a href="#video">{d.nav.video}</a>
          <a href="#telas">{d.nav.telas}</a>
          <a href="#claude">{d.nav.claude}</a>
          <a href="#instalacao">{d.nav.instalacao}</a>
          <a className="acao acao--fantasma" href={REPO} target="_blank" rel="noreferrer">
            {d.nav.github}
          </a>
        </nav>
        <Lingua />
      </header>

      {/* ── abertura ─────────────────────────────────────────────── */}
      <section className="secao heroi" id="inicio">
        <div className="secao__interno">
        <div className="painel heroi__painel">
        {/* A trama roda atrás do conteúdo, não em cima: é ambiente. */}
        <div className="heroi__fundo" aria-hidden>
          <Trama altura={760} />
        </div>
        <div className="heroi__interno">
        <motion.div
          className="heroi__texto"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease: [0.16, 1, 0.3, 1] }}
        >
          <span className="pilula" data-tom="acao">
            <span className="pilula__ponto" />
            {d.heroi.pilula}
          </span>

          <h1>
            {d.heroi.titulo1}
            <br className="so-largo" /> {d.heroi.titulo2}
          </h1>

          <p className="heroi__linha">{d.heroi.linha}</p>

          <Comando compacto />

          <div className="heroi__acoes">
            <a className="acao acao--fantasma" href="#como">
              {d.heroi.verComo}
            </a>
            <a className="acao acao--fantasma" href={REPO} target="_blank" rel="noreferrer">
              {d.heroi.lerCodigo}
            </a>
          </div>
        </motion.div>

        <Trilha postas={cargos} />
        </div>
        </div>
        </div>
      </section>

      {/* ── faixa ────────────────────────────────────────────────── */}
      {/* Uma faixa e não um banner: o que ela repete é a promessa do projeto,
          e uma frase que anda é lida por quem passa rolando. */}
      <div className="faixa">
        {/* A chave força a remontagem na troca de idioma: o laço mede a
            largura do texto uma vez, e sem isto a frase nova andaria com a
            medida da antiga. */}
        <TextLoop key={d.meta.lang} texto={d.faixa} />
      </div>

      {/* ── apresentação ─────────────────────────────────────────── */}
      <Secao id="video">
        <div className="painel video-painel">
          <div className="video-painel__texto">
            <h2>{d.video.titulo}</h2>
            <p>{d.video.texto}</p>
          </div>
          <Apresentacao />
        </div>
      </Secao>

      {/* ── por que existe ───────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>{d.porque.titulo}</h2>
          <p>{d.porque.texto}</p>
        </div>
        <div className="prosa">
          <p>{d.porque.p1}</p>
          <p>{d.porque.p2}</p>
        </div>
      </Secao>

      {/* ── como funciona ────────────────────────────────────────── */}
      <Secao id="como">
        <div className="secao__titulo">
          <h2>{d.como.titulo}</h2>
          <p>{d.como.texto}</p>
        </div>

        {/* Numeração aqui porque a seção É uma sequência: a ordem carrega a
            informação, e trocá-la mudaria o que o sistema faz. */}
        <ol className="passos">
          {d.como.passos.map((p, i) => (
            <li key={p.titulo}>
              <span className="passos__n num">{i + 1}</span>
              <div>
                <h3>{p.titulo}</h3>
                <p>{p.texto}</p>
              </div>
            </li>
          ))}
        </ol>
      </Secao>

      {/* ── a inversão do hardware ───────────────────────────────── */}
      <Secao>
        <div className="secao__titulo">
          <h2>{d.duelo.titulo}</h2>
          <p>{d.duelo.texto}</p>
        </div>

        <div className="painel">
        <div className="duelo">
          <div className="duelo__lado">
            <span className="duelo__nome mono">qwen3:14b</span>
            <span className="duelo__tipo">{d.duelo.denso}</span>
            <span className="duelo__peso">{d.duelo.discoA}</span>
            <div className="duelo__barra">
              <motion.span
                style={{ background: "var(--ink-3)" }}
                initial={{ width: 0 }}
                whileInView={{ width: "11%" }}
                viewport={{ once: true, margin: "-80px" }}
                transition={{ duration: 0.9, ease: [0.16, 1, 0.3, 1] }}
              />
            </div>
            <span className="duelo__valor num">0,6 tok/s</span>
          </div>

          <div className="duelo__lado" data-vence="true">
            <span className="duelo__nome mono">qwen3:30b-a3b</span>
            <span className="duelo__tipo">{d.duelo.moe}</span>
            <span className="duelo__peso">{d.duelo.discoB}</span>
            <div className="duelo__barra">
              <motion.span
                initial={{ width: 0 }}
                whileInView={{ width: "100%" }}
                viewport={{ once: true, margin: "-80px" }}
                transition={{ duration: 0.9, delay: 0.12, ease: [0.16, 1, 0.3, 1] }}
              />
            </div>
            <span className="duelo__valor num">5,7 tok/s</span>
          </div>
        </div>

        <p className="nota-medida">{d.duelo.nota}</p>
        </div>
      </Secao>

      {/* ── as telas, ao vivo ────────────────────────────────────── */}
      <Secao id="telas">
        <div className="secao__titulo">
          <h2>{d.telas.titulo}</h2>
          <p>{d.telas.texto}</p>
        </div>
        <Vitrine />
      </Secao>

      {/* ── o laço ───────────────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>{d.laco.titulo}</h2>
          <p>{d.laco.texto}</p>
        </div>
        <div className="prosa">
          <p>{d.laco.p1}</p>
          <p>{d.laco.p2}</p>
        </div>
      </Secao>

      {/* ── privacidade ──────────────────────────────────────────── */}
      <Secao>
        <div className="secao__titulo">
          <h2>{d.privacidade.titulo}</h2>
        </div>

        <div className="painel">
        <div className="fluxo">
          <div className="fluxo__col">
            <span className="fluxo__rotulo">{d.privacidade.sai}</span>
            <ul>
              {d.privacidade.saiItens.map((t) => (
                <li key={t}>{t}</li>
              ))}
            </ul>
          </div>
          <BorderGlow className="fluxo__col" raio={14}>
            <span className="fluxo__rotulo">{d.privacidade.fica}</span>
            <ul>
              {d.privacidade.ficaItens.map((t) => (
                <li key={t}>{t}</li>
              ))}
            </ul>
          </BorderGlow>
        </div>

        <p className="nota-medida">{d.privacidade.nota}</p>
        </div>
      </Secao>

      {/* ── Claude Code ──────────────────────────────────────────── */}
      {/* Fica depois da privacidade de propósito: é a única parte do produto
          que manda texto para fora, e a pessoa acabou de ler exatamente o que
          sai da máquina. Oferecer a troca antes disso seria vender a
          conveniência sem o custo. */}
      <Secao id="claude">
        <div className="secao__titulo">
          <span className="pilula pilula--claude">
            <MarcaClaude size={15} />
            {d.claude.pilula}
          </span>
          <h2>{d.claude.titulo}</h2>
          <p>{d.claude.texto}</p>
        </div>

        <div className="painel claude">
          {/* A arte veio como adesivo, com contorno branco e sombra. A máscara
              foi refeita a partir do que É desenho (o laranja do corpo e o
              preto dos olhos) em vez de tentar apagar o branco — assim some
              também o antialiasing da borda, que sobrevive a qualquer limiar
              de cor. Decorativa: quem usa leitor de tela já recebeu a marca no
              texto da pílula e no título. */}
          <img
            className="claude__mascote"
            src="claude-code.png"
            alt=""
            aria-hidden
            width={118}
            height={93}
            loading="lazy"
          />
          <div className="claude__grade">
            {d.claude.itens.map((it) => (
              <div className="claude__item" key={it.titulo}>
                <span className="claude__marca" aria-hidden>
                  <MarcaClaude size={17} />
                </span>
                <h3>{it.titulo}</h3>
                <p>{it.texto}</p>
              </div>
            ))}
          </div>

          <p className="nota-medida">{d.claude.nota}</p>
        </div>

        <p className="ressalva">{d.claude.ressalva}</p>
      </Secao>

      {/* ── instalação ───────────────────────────────────────────── */}
      <Secao id="instalacao" estreita>
        <div className="secao__titulo">
          <h2>{d.instalacao.titulo}</h2>
          <p>{d.instalacao.texto}</p>
        </div>

        <Comando />

        <div className="requisitos">
          {d.instalacao.requisitos.map((r) => (
            <div key={r.titulo}>
              <h3>{r.titulo}</h3>
              <p>{r.texto}</p>
            </div>
          ))}
        </div>
      </Secao>

      {/* ── open source ──────────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>{d.aberto.titulo}</h2>
        </div>
        <div className="prosa">
          <p>{d.aberto.p1}</p>
          <p>{d.aberto.p2}</p>
        </div>
        <div className="heroi__acoes">
          <a className="acao acao--forte" href={REPO} target="_blank" rel="noreferrer">
            {d.aberto.ver}
          </a>
          <a className="acao" href={`${REPO}/issues/new`} target="_blank" rel="noreferrer">
            {d.aberto.issue}
          </a>
        </div>
      </Secao>

      <footer className="rodape">
        <div className="rodape__marca">
          <Marca />
          <span>postly</span>
        </div>
        <p className="rodape__nota">
          {d.rodape.nota}{" "}
          <a href="https://github.com/JTeixeiraz" target="_blank" rel="noreferrer">
            JTeixeiraz
          </a>
        </p>
      </footer>
    </div>
  );
}

/** A marca: a haste do P é a rota, o ponto é a posta onde o despacho está. */
function Marca({ size = 22 }: { size?: number }) {
  return (
    <svg width={Math.round(size * (14 / 21.4))} height={size} viewBox="9 5.6 14 21.4" aria-hidden>
      <path
        d="M11.6 24.5V9.4h5.1a3.9 3.9 0 0 1 0 7.8h-5.1"
        stroke="currentColor"
        strokeWidth="2.6"
        fill="none"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="11.6" cy="17.2" r="2.5" fill="currentColor" />
    </svg>
  );
}
