import { useEffect, useRef, useState } from "react";
import { motion, useScroll, useSpring, useTransform } from "motion/react";
import Trilha from "./componentes/Trilha";
import Comando from "./componentes/Comando";
import Telas from "./componentes/Telas";

const REPO = "https://github.com/JTeixeiraz/Postly";

const CARGOS = [
  {
    cargo: "Diretor Geral",
    modelo: "qwen3:30b-a3b",
    nota: "Decide a estratégia entre as redes. Só existe quando há mais de uma.",
  },
  {
    cargo: "Gerente de Setor",
    modelo: "qwen3:30b-a3b",
    nota: "Um por rede. Lê o mercado pelo navegador e define a linha criativa.",
  },
  {
    cargo: "Criador",
    modelo: "gemma3:4b",
    nota: "Recebe briefing fechado e produz. Não decide nada, então não precisa raciocinar caro.",
  },
  {
    cargo: "Auditor",
    modelo: "qwen3:14b",
    nota: "Julga a peça pronta, com a imagem na frente. Reprovar devolve ao Criador.",
  },
];

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
      <span className="posta" data-passou={visivel} aria-hidden />
      {children}
    </section>
  );
}

export default function App() {
  const { scrollYProgress } = useScroll();
  // Mola em vez do valor cru: o scroll de trackpad chega em saltos, e a linha
  // seguindo o valor direto treme.
  const avanco = useSpring(scrollYProgress, { stiffness: 90, damping: 26, restDelta: 0.001 });
  const escala = useTransform(avanco, [0, 1], [0, 1]);

  return (
    <div className="pagina">
      <div className="espinha" aria-hidden>
        <div className="espinha__fio" />
        <motion.div className="espinha__aceso" style={{ scaleY: escala }} />
      </div>

      <header className="topo">
        <a className="topo__marca" href="#inicio">
          <Marca />
          <span>postly</span>
        </a>
        <nav className="topo__nav">
          <a href="#como">Como funciona</a>
          <a href="#telas">Telas</a>
          <a href="#instalacao">Instalação</a>
          <a className="btn btn--fantasma" href={REPO} target="_blank" rel="noreferrer">
            GitHub
          </a>
        </nav>
      </header>

      {/* ── abertura ─────────────────────────────────────────────── */}
      <section className="secao heroi" id="inicio">
        <motion.div
          className="heroi__texto"
          initial={{ opacity: 0, y: 18 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease: [0.16, 1, 0.3, 1] }}
        >
          <span className="pilula" data-tom="acao">
            <span className="pilula__ponto" />
            open source, sem fins lucrativos
          </span>

          <h1>
            Um departamento de marketing
            <br />
            que roda na sua máquina.
          </h1>

          <p className="heroi__linha">
            Quatro cargos de IA se revezam em modelos locais para pesquisar o mercado, criar a
            peça, auditar e publicar nas suas redes. Um modelo por vez, porque numa máquina
            comum não cabem quatro.
          </p>

          <Comando compacto />

          <div className="heroi__acoes">
            <a className="btn btn--fantasma" href="#como">
              Ver como funciona
            </a>
            <a className="btn btn--fantasma" href={REPO} target="_blank" rel="noreferrer">
              Ler o código
            </a>
          </div>
        </motion.div>

        <Trilha postas={CARGOS} />
      </section>

      {/* ── por que existe ───────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>Por que isto existe</h2>
          <p>
            Ferramentas de marketing com IA cobram por mês, guardam suas campanhas num servidor
            alheio e escondem qual modelo escreveu o quê.
          </p>
        </div>
        <div className="prosa">
          <p>
            O Postly faz o contrário: os modelos rodam no seu hardware, cada conversa fica em
            Markdown num arquivo seu, e o único custo variável é a geração de imagem.
          </p>
          <p>
            A parte difícil nunca foi chamar um modelo. Foi fazer quatro caberem numa máquina
            comum, e é disso que trata o middleware por trás do revezamento.
          </p>
        </div>
      </Secao>

      {/* ── como funciona ────────────────────────────────────────── */}
      <Secao id="como">
        <div className="secao__titulo">
          <h2>Um modelo por vez</h2>
          <p>
            Um agente de marketing precisa de raciocínio para decidir e de obediência para
            executar, e esses são modelos diferentes. Rodar todos ao mesmo tempo não cabe.
          </p>
        </div>

        {/* Numeração aqui porque a seção É uma sequência: a ordem carrega a
            informação, e trocá-la mudaria o que o sistema faz. */}
        <ol className="passos">
          <li>
            <span className="passos__n num">1</span>
            <div>
              <h3>Mede a memória</h3>
              <p>
                Antes de cada turno o sistema lê quanta RAM está livre naquele instante, não
                quanta a máquina tem.
              </p>
            </div>
          </li>
          <li>
            <span className="passos__n num">2</span>
            <div>
              <h3>Escolhe o mais forte que couber</h3>
              <p>
                Dentro do nível daquele cargo. Se nada couber, ele rebaixa o nível e avisa no
                relatório em vez de falhar.
              </p>
            </div>
          </li>
          <li>
            <span className="passos__n num">3</span>
            <div>
              <h3>Grava a conversa inteira</h3>
              <p>
                System prompt, entrada, resposta completa e o raciocínio, num arquivo por turno.
                É o que permite auditar depois.
              </p>
            </div>
          </li>
          <li>
            <span className="passos__n num">4</span>
            <div>
              <h3>Descarrega e passa adiante</h3>
              <p>
                Só a mensagem que atravessa segue para o próximo cargo. Nunca há dois modelos
                residentes ao mesmo tempo.
              </p>
            </div>
          </li>
        </ol>
      </Secao>

      {/* ── a inversão do hardware ───────────────────────────────── */}
      <Secao>
        <div className="secao__titulo">
          <h2>O catálogo ranqueia por velocidade, não por tamanho</h2>
          <p>
            A escolha muda com o hardware, e num PC sem GPU ela inverte a intuição de quem olha
            só o tamanho do arquivo.
          </p>
        </div>

        <div className="duelo">
          <div className="duelo__lado">
            <span className="duelo__nome mono">qwen3:14b</span>
            <span className="duelo__tipo">denso · 14B ativos</span>
            <span className="duelo__peso">9,3 GB em disco</span>
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
            <span className="duelo__tipo">MoE · 3B ativos de 30B</span>
            <span className="duelo__peso">19 GB em disco</span>
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

        <p className="nota-medida">
          Medido nesta máquina de desenvolvimento: Ryzen 5000, sem GPU utilizável. O modelo que
          ocupa o dobro de memória gera quase dez vezes mais rápido, porque só os especialistas
          ativos passam pela CPU. Otimizar por tamanho de arquivo leva à escolha errada.
        </p>
      </Secao>

      {/* ── as telas ─────────────────────────────────────────────── */}
      <Secao id="telas">
        <div className="secao__titulo">
          <h2>O que você abre</h2>
          <p>Capturas do aplicativo rodando. Nenhum mockup.</p>
        </div>
        <Telas />
      </Secao>

      {/* ── o laço ───────────────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>O desempenho volta para dentro</h2>
          <p>
            Um gerador de conteúdo sem retorno de desempenho repete o que o modelo acha bonito,
            não o que funcionou.
          </p>
        </div>
        <div className="prosa">
          <p>
            Você registra o resultado de cada publicação e o sistema ranqueia as peças contra a
            mediana da própria conta. Essa leitura entra no prompt do cargo que decide a próxima
            campanha, com o número exato a bater.
          </p>
          <p>
            A regra é sempre superar e nunca repetir: o que rendeu vira piso, não molde. A única
            exceção é o acerto extraordinário, quando uma peça bate três vezes a mediana — aí
            deixou de ser sorte, e vale seguir naquela linha enquanto ela render.
          </p>
        </div>
      </Secao>

      {/* ── privacidade ──────────────────────────────────────────── */}
      <Secao>
        <div className="secao__titulo">
          <h2>O que sai da sua máquina</h2>
        </div>

        <div className="fluxo">
          <div className="fluxo__col">
            <span className="fluxo__rotulo">sai</span>
            <ul>
              <li>O prompt da imagem, para o serviço de arte que você escolheu.</li>
              <li>O tráfego do navegador para as redes onde você publica, na sua sessão.</li>
            </ul>
          </div>
          <div className="fluxo__col" data-fica="true">
            <span className="fluxo__rotulo">fica</span>
            <ul>
              <li>Os modelos e tudo que eles escrevem.</li>
              <li>O grafo de contexto, as campanhas e as transcrições.</li>
              <li>As credenciais, cifradas em AES-256-GCM.</li>
            </ul>
          </div>
        </div>

        <p className="nota-medida">
          Não há telemetria, não há servidor do projeto, não há conta para criar. Sobre o cofre,
          uma ressalva que costuma ser vendida com mais confiança do que merece: ele protege
          contra backup, sincronização de nuvem e alguém lendo o disco. Não protege contra um
          programa rodando com o seu usuário.
        </p>
      </Secao>

      {/* ── instalação ───────────────────────────────────────────── */}
      <Secao id="instalacao" estreita>
        <div className="secao__titulo">
          <h2>Instalação</h2>
          <p>
            Um comando. Ele detecta o sistema, confirma com você e baixa o pacote da versão mais
            recente.
          </p>
        </div>

        <Comando />

        <div className="requisitos">
          <div>
            <h3>Ollama</h3>
            <p>
              Você não precisa instalar antes. O app faz isso na primeira abertura, com barra de
              progresso, usando pacman, winget, Homebrew ou o script oficial.
            </p>
          </div>
          <div>
            <h3>Uma chave de imagem</h3>
            <p>
              Gemini, OpenAI, FLUX, Stability AI ou Higgsfield. Só a arte precisa de serviço
              externo; o texto todo sai dos modelos locais.
            </p>
          </div>
          <div>
            <h3>Memória</h3>
            <p>
              16 GB para um uso confortável. Roda em 8 GB com modelos menores, e o catálogo
              mostra quais cabem antes de você baixar.
            </p>
          </div>
        </div>
      </Secao>

      {/* ── open source ──────────────────────────────────────────── */}
      <Secao estreita>
        <div className="secao__titulo">
          <h2>Sem fins lucrativos</h2>
        </div>
        <div className="prosa">
          <p>
            O Postly é gratuito e licenciado sob MIT. Não há versão paga, plano, assinatura,
            nem intenção de criar uma. O código inteiro está no GitHub, incluindo o do cofre
            que guarda suas credenciais, porque um app que pede chave de API precisa ser
            auditável.
          </p>
          <p>
            Se algo aqui foi útil, o retorno que faz diferença é uma issue com um problema
            reproduzível ou um adaptador de rede social que parou de funcionar.
          </p>
        </div>
        <div className="heroi__acoes">
          <a className="btn btn--acao" href={REPO} target="_blank" rel="noreferrer">
            Ver no GitHub
          </a>
          <a
            className="btn"
            href={`${REPO}/issues/new`}
            target="_blank"
            rel="noreferrer"
          >
            Abrir uma issue
          </a>
        </div>
      </Secao>

      <footer className="rodape">
        <div className="rodape__marca">
          <Marca />
          <span>postly</span>
        </div>
        <p className="rodape__nota">
          MIT · feito por{" "}
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
