import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";

interface Tela {
  id: string;
  arquivo: string;
  nome: string;
  texto: string;
  alt: string;
}

/** As oito telas reais do aplicativo.
 *
 *  Capturas do app rodando, não mockup: o que aparece aqui é literalmente o
 *  que a pessoa abre depois de instalar. Um mockup bonito que não corresponde
 *  ao produto é a forma mais rápida de queimar confiança na primeira abertura. */
const TELAS: Tela[] = [
  {
    id: "preparacao",
    arquivo: "preparacao.png",
    nome: "Preparação",
    texto:
      "O app mede a máquina antes de pedir qualquer coisa: memória livre, acelerador, se o Ollama está no ar. Se não estiver, um botão instala.",
    alt: "Tela de preparação mostrando 30,7 GB instalados, 15,5 GB livres e o teto de 21,5 GB por modelo",
  },
  {
    id: "modelos",
    arquivo: "modelos.png",
    nome: "Modelos",
    texto:
      "Trinta e sete modelos em nove famílias, ranqueados pela velocidade que teriam nesta máquina. O botão de baixar funciona: todas as tags existem na biblioteca pública do Ollama.",
    alt: "Catálogo de modelos com filtro por família e velocidade estimada em tokens por segundo",
  },
  {
    id: "imagem",
    arquivo: "imagem.png",
    nome: "Gerador de arte",
    texto:
      "Cinco serviços de imagem integrados, escolhidos clicando na logo. Cada um guarda a própria chave, e o cartão diz quais foram testados contra a API real.",
    alt: "Escolha do gerador de imagem entre Gemini, OpenAI, FLUX, Stability AI e Higgsfield",
  },
  {
    id: "campanha",
    arquivo: "campanha.png",
    nome: "Campanha",
    texto:
      "Você escreve o objetivo em uma frase. O painel da direita responde antes de começar: quantos turnos, quantas imagens e quanto tempo, estimado pela velocidade medida aqui.",
    alt: "Tela de campanha com o objetivo, as redes escolhidas e o plano de execução",
  },
  {
    id: "cerebro",
    arquivo: "cerebro.png",
    nome: "Cérebro",
    texto:
      "O contexto compartilhado é um grafo ponderado, sem banco de dados. Arraste os nodes, aproxime com a roda, clique para ver a vizinhança ordenada que um agente recebe ao consultar.",
    alt: "Grafo de conhecimento com nodes arrastáveis e pesos nas arestas",
  },
  {
    id: "auditoria",
    arquivo: "auditoria.png",
    nome: "Auditoria",
    texto:
      "O desempenho de cada publicação, ranqueado contra a mediana da própria conta. É daqui que sai a ordem para a próxima campanha.",
    alt: "Auditoria de desempenho com o ranking das publicações e o veredito por rede",
  },
  {
    id: "historico",
    arquivo: "historico.png",
    nome: "Histórico",
    texto:
      "Cada campanha vira uma pasta sua: as peças, as artes e a conversa inteira de cada cargo em Markdown. Nada depende do app estar aberto.",
    alt: "Histórico com a galeria das peças produzidas e o estado de publicação de cada uma",
  },
  {
    id: "referencias",
    arquivo: "referencias.png",
    nome: "Referências",
    texto:
      "Material da sua marca vai como imagem para os modelos que enxergam. Referência de estilo vai só como texto, para não sair logotipo alheio na sua peça.",
    alt: "Tela de referências e identidade visual da marca",
  },
];

export default function Telas() {
  const [atual, setAtual] = useState(0);
  const tela = TELAS[atual];

  return (
    <div className="telas">
      <nav className="telas__abas" aria-label="Telas do aplicativo">
        {TELAS.map((t, i) => (
          <button
            key={t.id}
            className="telas__aba"
            aria-current={i === atual ? "true" : undefined}
            onClick={() => setAtual(i)}
          >
            {i === atual && (
              <motion.span
                className="telas__bolha"
                layoutId="aba-tela"
                transition={{ type: "spring", stiffness: 400, damping: 33 }}
              />
            )}
            <span>{t.nome}</span>
          </button>
        ))}
      </nav>

      <div className="telas__palco">
        <AnimatePresence mode="wait">
          <motion.figure
            key={tela.id}
            className="telas__quadro"
            initial={{ opacity: 0, y: 14 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.34, ease: [0.16, 1, 0.3, 1] }}
          >
            <img
              src={`./capturas/${tela.arquivo}`}
              alt={tela.alt}
              width={1440}
              height={1000}
              // A primeira carrega junto com a página; as outras só quando a
              // pessoa troca de aba.
              loading={atual === 0 ? "eager" : "lazy"}
            />
            <figcaption>{tela.texto}</figcaption>
          </motion.figure>
        </AnimatePresence>
      </div>
    </div>
  );
}
