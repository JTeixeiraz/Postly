import { useCallback, useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { api } from "./api";
import { formatarBytes, useIdioma } from "./i18n";
import type { Diagnostico } from "./types";
import Lang from "./components/Lang";
import Marca from "./components/Marca";
import GuiaTopo from "./components/GuiaTopo";
import type { Passo } from "./components/Guia";
import Tour from "./components/Tour";
import Skills from "./components/Skills";
import Tema from "./components/Tema";
import ModalAtualizacao from "./components/ModalAtualizacao";
import ModalFalha from "./components/ModalFalha";
import ModalLimite from "./components/ModalLimite";
import ModalMotion from "./components/ModalMotion";
import ModalNarracao from "./components/ModalNarracao";
import Auditoria from "./screens/Auditoria";
import Meter from "./components/Meter";
import {
  IconArchive,
  IconGauge,
  IconChip,
  IconGraph,
  IconLayers,
  IconRelay,
  IconFilm,
} from "./components/Icons";
import Preparo from "./screens/Preparo";
import Modelos from "./screens/Modelos";
import Campanha from "./screens/Campanha";
import Video from "./screens/Video";
import Cerebro from "./screens/Cerebro";
import Historico from "./screens/Historico";

type Tela = "preparo" | "modelos" | "campanha" | "video" | "cerebro" | "auditoria" | "historico";

const TELAS: { id: Tela; icone: typeof IconChip }[] = [
  { id: "preparo", icone: IconChip },
  { id: "modelos", icone: IconLayers },
  { id: "campanha", icone: IconRelay },
  // O video fica depois da campanha e antes do cerebro: e a outra coisa que a
  // pessoa PEDE ao sistema, e as duas telas seguintes sao ferramentas de ler o
  // que ja aconteceu, nao de pedir coisa nova.
  { id: "video", icone: IconFilm },
  { id: "cerebro", icone: IconGraph },
  // A auditoria fica depois do cerebro e antes do historico: ela le o que ja
  // foi publicado, entao so faz sentido quando ha campanha atras.
  { id: "auditoria", icone: IconGauge },
  { id: "historico", icone: IconArchive },
];

export default function App() {
  const { d, idioma } = useIdioma();
  const [tela, setTela] = useState<Tela>("preparo");
  const [diag, setDiag] = useState<Diagnostico | null>(null);
  const [liberado, setLiberado] = useState(0);

  const recarregar = useCallback(async () => {
    setDiag(await api.diagnostico());
  }, []);

  // Estado de ativacao: o que ja esta pronto para a primeira campanha rodar.
  // Vive aqui, e nao dentro do guia, porque as telas mudam esse estado (baixar
  // um modelo, salvar a chave) e o trilho precisa reagir a isso.
  const [temModelo, setTemModelo] = useState(false);
  const [temChave, setTemChave] = useState(false);

  const conferirAtivacao = useCallback(async () => {
    const [cat, cofre] = await Promise.all([
      api.catalogoModelos().catch(() => []),
      api.resumoCofre().catch(() => null),
    ]);
    setTemModelo(cat.some((m) => m.installed));
    setTemChave(!!cofre?.has_gemini_key);
  }, []);

  useEffect(() => {
    void conferirAtivacao();
    // O guia precisa acompanhar o que acontece nas telas; reconferir na troca
    // de tela e barato e evita um barramento de eventos so para isto.
  }, [conferirAtivacao, tela]);

  useEffect(() => {
    void recarregar().catch(() => {});
  }, [recarregar]);

  // A memória muda debaixo da pessoa enquanto ela decide, e a decisão seguinte
  // depende dela. Só a leitura de RAM é reamostrada: sondar o acelerador a cada
  // poucos segundos custaria um processo externo por ciclo.
  useEffect(() => {
    const t = setInterval(() => {
      void api
        .memoria()
        .then((ram) =>
          setDiag((atual) =>
            atual ? { ...atual, computacao: { ...atual.computacao, ram } } : atual
          )
        )
        .catch(() => {});
    }, 5000);
    return () => clearInterval(t);
  }, []);

  // O portão existe só para guiar a primeira execução. Chegar na campanha
  // significa que o preparo terminou, e a partir daí ele sai da frente: cérebro
  // e histórico são ferramentas, não etapas.
  //
  // TODA navegação passa por aqui, inclusive o clique na aba. Antes o botão
  // "Continuar" chamava esta função e a aba chamava `setTela` direto, então
  // quem andava pelo cabeçalho chegava na tela sem mover o portão — e era
  // obrigado a voltar e apertar o botão para destravar o que já tinha
  // alcançado. Dois caminhos para o mesmo lugar precisam fazer a mesma coisa.
  const avancar = (destino: Tela) => {
    setLiberado((n) =>
      destino === "campanha"
        ? TELAS.length - 1
        : Math.max(n, TELAS.findIndex((t) => t.id === destino))
    );
    setTela(destino);
  };

  const rotulos: Record<Tela, string> = {
    preparo: d.nav.prep,
    modelos: d.nav.models,
    campanha: d.nav.campaign,
    video: d.nav.video,
    cerebro: d.nav.brain,
    auditoria: d.motion.nav,
    historico: d.nav.history,
  };

  const ram = diag?.computacao.ram;

  const passos: Passo[] = [
    { id: "maquina", rotulo: d.guide.machine, nota: d.guide.machineNote, feito: !!diag },
    {
      id: "ollama",
      rotulo: d.guide.ollama,
      feito: diag?.ollama?.state === "pronto",
      ir: () => setTela("preparo"),
    },
    {
      id: "modelo",
      rotulo: d.guide.model,
      nota: d.guide.modelNote,
      feito: temModelo,
      ir: () => avancar("modelos"),
    },
    {
      id: "chave",
      rotulo: d.guide.key,
      nota: d.guide.keyNote,
      feito: temChave,
      ir: () => avancar("campanha"),
    },
  ];

  return (
    <div className="shell">
      {/* Fora do fluxo das telas: a pergunta chega no meio de qualquer uma
          delas, e a campanha fica parada ate a resposta. */}
      <ModalMotion />
      <ModalNarracao />
      <ModalLimite />
      <ModalFalha />
      <ModalAtualizacao />
      <header className="topo">
        <div className="marca">
          <Marca size={23} />
          <span className="marca__nome">postly</span>
        </div>

        <nav className="abas">
          {TELAS.map(({ id, icone: Icone }, i) => {
            const bloqueado = i > liberado + 1;
            const atual = tela === id;
            return (
              <button
                key={id}
                className="aba"
                aria-current={atual ? "page" : undefined}
                disabled={bloqueado}
                title={rotulos[id]}
                onClick={() => avancar(id)}
              >
                {/* A pilula ativa e um elemento so, compartilhado entre as
                    abas: o layoutId faz ela deslizar de uma para outra em vez
                    de sumir aqui e aparecer ali. */}
                {atual && (
                  <motion.span
                    className="aba__bolha"
                    layoutId="aba-ativa"
                    transition={{ type: "spring", stiffness: 380, damping: 32 }}
                  />
                )}
                <Icone size={15} />
                <span className="aba__rot">{rotulos[id]}</span>
                {id === "preparo" && diag?.ollama?.state && diag.ollama.state !== "pronto" && (
                  <span className="aba__badge" aria-hidden />
                )}
              </button>
            );
          })}
        </nav>

        <div className="topo__acoes">
          <GuiaTopo passos={passos} />
          {ram && (
            <div className="ram-topo" title={diag?.computacao.mode_label}>
              <Meter ram={ram} />
              <span className="ram-topo__txt num">
                {formatarBytes(ram.available_bytes, idioma)} {d.common.free}
              </span>
            </div>
          )}
          <Skills />
          <Tour />
          <Tema />
          <Lang />
        </div>
      </header>

      <main className="main" key={tela}>
        <AnimatePresence mode="wait">
          <motion.div
            key={tela}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -6 }}
            transition={{ type: "spring", stiffness: 260, damping: 30 }}
            className="page"
          >
            {tela === "preparo" && (
              <Preparo diag={diag} recarregar={recarregar} avancar={() => avancar("modelos")} />
            )}
            {tela === "modelos" && (
              <Modelos diag={diag} avancar={() => avancar("campanha")} />
            )}
            {tela === "campanha" && <Campanha diag={diag} />}
            {tela === "video" && <Video />}
            {tela === "cerebro" && <Cerebro />}
            {tela === "auditoria" && <Auditoria />}
            {tela === "historico" && <Historico />}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
