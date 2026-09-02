import { useCallback, useEffect, useState, type ReactNode } from "react";
import { motion } from "motion/react";
import { api, ouvirProvisao } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { Diagnostico, OllamaStatus, StatusNavegador, PerfilComputacao, ProgressoProvisao, RamSnapshot, SondaSistema, Vaga } from "../types";
import Meter from "../components/Meter";
import Elenco from "../components/Elenco";
import Otimizar from "./Otimizar";
import { IconAlert, IconArrow, IconCheck, IconDot, IconSpinner } from "../components/Icons";
import { useOuvinte } from "../ouvir";

type Estado = "espera" | "rodando" | "ok" | "aviso" | "falhou";
type Chave = "sistema" | "memoria" | "acelerador" | "ollama" | "navegador";

interface Linha {
  estado: Estado;
  resultado?: ReactNode;
  acao?: ReactNode;
}

const ICONE: Record<Estado, typeof IconDot> = {
  espera: IconDot,
  rodando: IconSpinner,
  ok: IconCheck,
  aviso: IconAlert,
  falhou: IconAlert,
};

interface Props {
  diag: Diagnostico | null;
  recarregar: () => Promise<void>;
  avancar: () => void;
}

const CHAVE_PREPARO = "postly:preparo-feito";

export default function Preparo({ diag, recarregar, avancar }: Props) {
  const { d, f, idioma } = useIdioma();
  const [linhas, setLinhas] = useState<Record<Chave, Linha>>({
    sistema: { estado: "espera" },
    memoria: { estado: "espera" },
    acelerador: { estado: "espera" },
    ollama: { estado: "espera" },
    navegador: { estado: "espera" },
  });
  const [rodando, setRodando] = useState(false);
  const [terminou, setTerminou] = useState(false);
  const [ram, setRam] = useState<RamSnapshot | null>(null);
  const [ollama, setOllama] = useState<OllamaStatus | null>(null);
  const [navegador, setNavegador] = useState<StatusNavegador | null>(null);
  const [autoInstalando, setAutoInstalando] = useState(false);
  // Guardado no navegador embutido e não no cofre: é preferência de fluxo, não
  // segredo, e o cofre exige decifrar a cada leitura.
  const [primeiraVez] = useState(() => localStorage.getItem(CHAVE_PREPARO) !== "1");
  const [ocupado, setOcupado] = useState<string | null>(null);
  /** Andamento da instalacao do Ollama. Null quando nao ha instalacao rodando. */
  const [provisao, setProvisao] = useState<ProgressoProvisao | null>(null);
  const [mostrarLimpeza, setMostrarLimpeza] = useState(false);
  const [vagas, setVagas] = useState<Vaga[] | null>(null);

  const marcar = (chave: Chave, linha: Linha) =>
    setLinhas((atual) => ({ ...atual, [chave]: linha }));

  const verificar = useCallback(async () => {
    setRodando(true);
    setTerminou(false);
    setMostrarLimpeza(false);
    setLinhas({
      sistema: { estado: "espera" },
      memoria: { estado: "espera" },
      acelerador: { estado: "espera" },
      ollama: { estado: "espera" },
    navegador: { estado: "espera" },
    });

    // Sistema operacional.
    marcar("sistema", { estado: "rodando" });
    const sis: SondaSistema = await api.sondaSistema();
    marcar("sistema", {
      estado: "ok",
      resultado: f(d.boot.osOk, { os: sis.plataforma_label }),
    });

    // Memória.
    marcar("memoria", { estado: "rodando" });
    const mem = await api.sondaMemoria();
    setRam(mem);
    marcar("memoria", {
      estado: mem.low_ram_warning ? "aviso" : "ok",
      resultado: mem.low_ram_warning
        ? f(d.boot.memoryLow, { free: formatarBytes(mem.available_bytes, idioma) })
        : f(d.boot.memoryOk, {
            total: formatarBytes(mem.total_bytes, idioma),
            free: formatarBytes(mem.available_bytes, idioma),
          }),
      acao: mem.low_ram_warning ? (
        <button className="btn btn--sm" onClick={() => setMostrarLimpeza(true)}>
          {d.boot.optimize}
        </button>
      ) : undefined,
    });

    // Acelerador: a sonda mais cara, porque dispara processos externos.
    marcar("acelerador", { estado: "rodando" });
    const perfil: PerfilComputacao = await api.sondaAcelerador();
    marcar("acelerador", {
      estado: perfil.mode === "cpu" ? "aviso" : "ok",
      resultado:
        perfil.mode === "dedicada"
          ? f(d.boot.accelGpu, {
              name: perfil.primary_name ?? "GPU",
              vram: formatarBytes(perfil.vram_total_bytes, idioma),
            })
          : perfil.mode === "unificada"
            ? f(d.boot.accelUnified, { name: perfil.primary_name ?? "GPU" })
            : d.boot.accelNone,
    });

    // Ollama.
    marcar("ollama", { estado: "rodando" });
    const olla = await api.sondaOllama();
    setOllama(olla);
    aplicarOllama(olla);

    // Navegador. Vem depois do Ollama porque só importa se houver o que
    // publicar, mas é igualmente obrigatório: sem Chromium não há publicação
    // nem coleta de desempenho.
    marcar("navegador", { estado: "rodando" });
    // A sonda roda um processo Node. Se ele não subir, o retorno vem vazio, e
    // tratar isso como "não instalado" é mais honesto do que derrubar a tela.
    const nav = await api.sondaNavegador().catch(() => null);
    setNavegador(nav);
    aplicarNavegador(
      nav ?? {
        state: "ausente",
        caminho: null,
        detalhe: d.boot.browserUnknown,
      }
    );

    // Fecha a leitura mostrando a consequência dela: quem seria escalado agora.
    setVagas(await api.elenco());

    setRodando(false);
    setTerminou(true);

    // Primeira abertura: instala o que falta sem esperar clique.
    //
    // A tela antes só detectava e oferecia um botão para cada peça. Mas
    // ninguém abre um aplicativo de marketing querendo administrar
    // dependências: o Ollama executa os modelos e o navegador publica, e
    // faltando qualquer um o produto não faz nada. Perguntar "deseja instalar
    // o que é obrigatório?" é dar uma escolha que não existe.
    //
    // Só na PRIMEIRA vez. Depois disso, se algo sumir, é caso de decidir — o
    // botão volta e a instalação vira gesto explícito.
    if (primeiraVez) {
      await instalarOQueFalta(olla, nav);
    }
    void recarregar();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [d, f, idioma, recarregar]);

  /** Instala Ollama e navegador, em sequência, sem pedir clique.
   *
   *  Em sequência e não em paralelo: os dois baixam centenas de megabytes, e
   *  dividir a banda faria os dois demorarem o dobro, com duas barras
   *  disputando a mesma linha da tela.
   *
   *  O Ollama vem primeiro porque é o maior e o mais provável de faltar; se
   *  ele falhar, o navegador ainda é tentado — uma campanha sem publicação
   *  automática continua produzindo peças, mas sem modelo não há nada. */
  const instalarOQueFalta = async (
    olla: OllamaStatus,
    nav: StatusNavegador | null
  ) => {
    const faltaOllama = olla.state !== "pronto";
    const faltaNavegador = nav?.state !== "pronto" && nav?.state !== "semnode";
    if (!faltaOllama && !faltaNavegador) {
      // Nada a fazer, e a marca de primeira vez já pode cair.
      localStorage.setItem(CHAVE_PREPARO, "1");
      return;
    }

    setAutoInstalando(true);
    const ollamaOk = faltaOllama ? await prepararOllama() : true;
    const navegadorOk = faltaNavegador ? await prepararNavegador() : true;
    setAutoInstalando(false);
    // A MARCA SO CAI QUANDO DEU CERTO. Antes ela caia sempre, e uma tentativa que
    // falhava queimava a unica chance automatica: na abertura seguinte `primeiraVez`
    // ja era falso e o app nunca mais tentava sozinho — a pessoa ficava com o erro
    // na tela e um botao para sempre.
    //
    // Foi assim que um usuario no Windows ficou preso: o `sidecar/` nao vinha no
    // instalador, a instalacao falhava, e a segunda abertura ja nem tentava. Aquela
    // causa esta corrigida em `recursos.rs`; isto conserta o que acontece quando a
    // instalacao falha por outro motivo — sem rede, disco cheio, npm ausente.
    if (ollamaOk && navegadorOk) {
      localStorage.setItem(CHAVE_PREPARO, "1");
    }
  };

  const aplicarOllama = (olla: OllamaStatus) => {
    if (olla.state === "pronto") {
      marcar("ollama", {
        estado: "ok",
        resultado: f(d.boot.ollamaReady, { version: olla.version ?? "?" }),
      });
      return;
    }
    marcar("ollama", {
      estado: "falhou",
      // O Ollama nao e opcional: sem ele nao ha modelo para rodar. A frase diz
      // isso antes de a pessoa procurar o botao de pular.
      resultado: `${olla.state === "ausente" ? d.boot.ollamaMissing : d.boot.ollamaStopped} ${d.boot.installRequired}`,
      acao: (
        <button className="btn btn--sm btn--key" onClick={prepararOllama}>
          {olla.state === "ausente" ? d.boot.ollamaInstall : d.boot.ollamaStart}
        </button>
      ),
    });
  };

  const aplicarNavegador = (nav: StatusNavegador) => {
    if (nav.state === "pronto") {
      marcar("navegador", { estado: "ok", resultado: d.boot.browserReady });
      return;
    }
    // "sem node" é o único caso que o app não resolve sozinho: instalar um
    // runtime na máquina de alguém sem pedir seria invasivo.
    const podeInstalar = nav.state !== "semnode";
    marcar("navegador", {
      estado: "falhou",
      resultado: nav.detalhe,
      acao: podeInstalar ? (
        <button className="btn btn--sm btn--key" onClick={prepararNavegador}>
          {d.boot.browserInstall}
        </button>
      ) : undefined,
    });
  };

  const prepararNavegador = async (): Promise<boolean> => {
    setOcupado("navegador");
    marcar("navegador", { estado: "rodando", resultado: d.boot.browserDownloading });
    try {
      const r = await api.provisionarNavegador();
      setNavegador(r.status_final);
      if (r.ok) {
        marcar("navegador", { estado: "ok", resultado: d.boot.browserReady });
        return true;
      } else {
        marcar("navegador", {
          estado: "falhou",
          resultado: r.erros.join(" · ") || r.status_final.detalhe,
          acao: (
            <button className="btn btn--sm" onClick={prepararNavegador}>
              {d.common.retry}
            </button>
          ),
        });
      }
      return false;
    } finally {
      setOcupado(null);
    }
  };

  // O instalador do Ollama baixa perto de um giga. Sem acompanhar linha a
  // linha, a tela ficaria minutos parada e a pessoa nao saberia se travou.
  useOuvinte(() => ouvirProvisao(setProvisao), []);

  const prepararOllama = async (): Promise<boolean> => {
    setOcupado("ollama");
    setProvisao({ passo: 0, total: 0, label: d.boot.installing, linha: "", percent: 0, fase: "instalando" });
    marcar("ollama", { estado: "rodando", resultado: d.boot.probing });
    try {
      const r = await api.provisionarOllama();
      setProvisao(null);
      const novo = await api.sondaOllama();
      setOllama(novo);
      if (novo.state === "pronto") {
        marcar("ollama", {
          estado: "ok",
          resultado: f(d.boot.ollamaReady, { version: novo.version ?? "?" }),
        });
        void recarregar();
        return true;
      } else {
        marcar("ollama", {
          estado: "falhou",
          resultado: r.errors.join(" · ") || d.boot.ollamaMissing,
          acao: (
            <button className="btn btn--sm" onClick={prepararOllama}>
              {d.common.retry}
            </button>
          ),
        });
      }
      return false;
    } finally {
      setOcupado(null);
    }
  };

  // Se a máquina já foi lida numa sessão anterior, não obriga a repetir.
  useEffect(() => {
    if (diag && !terminou && !rodando && linhas.sistema.estado === "espera") {
      void verificar();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [diag]);

  // Os dois são obrigatórios: o Ollama executa os modelos, o navegador
  // publica. Faltando qualquer um, metade do produto não existe — e liberar a
  // próxima tela seria empurrar a pessoa para uma campanha que trava no fim.
  const pronto = ollama?.state === "pronto" && navegador?.state === "pronto";
  const ordem: Chave[] = ["sistema", "memoria", "acelerador", "ollama", "navegador"];
  const titulos: Record<Chave, string> = {
    sistema: d.boot.os,
    memoria: d.boot.memory,
    acelerador: d.boot.accel,
    ollama: d.boot.ollama,
    navegador: d.boot.browserTitle,
  };

  return (
    <>
      <header className="page__head">
        <h1>{d.boot.title}</h1>
        <p>{d.boot.lead}</p>
      </header>

      <section className="card">
        <div className="card">
          {ordem.map((chave) => {
            const linha = linhas[chave];
            const Icone = ICONE[linha.estado];
            return (
              <div className="sonda" key={chave} data-estado={linha.estado}>
                <span className="sonda__icone">
                  <Icone size={17} />
                </span>
                <div>
                  <div className="sonda__titulo">{titulos[chave]}</div>
                  <motion.div
                    className="sonda__resultado"
                    key={String(linha.resultado)}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.24, ease: [0.16, 1, 0.3, 1] }}
                  >
                    {linha.estado === "rodando"
                      ? d.boot.probing
                      : (linha.resultado ?? "—")}
                  </motion.div>

                  {chave === "memoria" && ram && (
                    <div className="stack stack--tight" style={{ marginTop: 12, maxWidth: 460 }}>
                      <Meter ram={ram} />
                      <span className="hint num">
                        {formatarBytes(ram.max_budget_bytes, idioma)} {d.boot.budgetCap}
                      </span>
                    </div>
                  )}
                </div>
                <div>{ocupado === chave ? null : linha.acao}</div>
              </div>
            );
          })}
        </div>

        {/* Na primeira abertura o app instala sozinho o que falta, e isso
            leva minutos. Sem esta linha a tela fica parada com sondas
            piscando e nada explica por quê. */}
        {autoInstalando && (
          <div className="note" data-tone="signal" style={{ marginTop: 18 }}>
            <strong>{d.boot.autoTitle}</strong>
            <span>{d.boot.autoBody}</span>
          </div>
        )}

        {/* Progresso real da instalacao: passo, percentual quando o
            instalador informa, e a ultima linha que ele escreveu. */}
        {provisao && (
          <div className="provisao">
            <div className="provisao__topo">
              <span className="provisao__nome">
                {provisao.fase === "subindo" ? d.boot.installServer : d.boot.installing}
              </span>
              {provisao.total > 0 && (
                <span className="hint num">
                  {f(d.boot.installStep, { n: provisao.passo, t: provisao.total })}
                </span>
              )}
              {provisao.percent !== null && (
                <span className="provisao__pct num">{provisao.percent.toFixed(0)}%</span>
              )}
            </div>
            <div className="provisao__barra" data-indeterminado={provisao.percent === null}>
              <span style={provisao.percent !== null ? { width: `${provisao.percent}%` } : undefined} />
            </div>
            {provisao.linha && <span className="provisao__linha mono">{provisao.linha}</span>}
          </div>
        )}

        <div className="row" style={{ marginTop: 20 }}>
          <button className="btn" onClick={verificar} disabled={rodando}>
            {rodando ? d.boot.probing : terminou ? d.boot.rerun : d.boot.run}
          </button>
          {!mostrarLimpeza && terminou && (
            <button className="btn btn--quiet btn--sm" onClick={() => setMostrarLimpeza(true)}>
              {d.boot.optimize}
            </button>
          )}
          <span className="push" />
          <button className="btn btn--key" onClick={avancar} disabled={!pronto}>
            {d.common.continue}
            <IconArrow size={16} />
          </button>
        </div>
        {!pronto && terminou && (
          <p className="hint" style={{ textAlign: "right", marginTop: 8 }}>
            {d.boot.blocked}
          </p>
        )}
      </section>

      {vagas && (
        <section className="card">
          <div className="card__topo">
            <h2>{d.boot.crew}</h2>
          </div>
          <p className="hint" style={{ marginBottom: 18 }}>{d.boot.crewLead}</p>
          <Elenco vagas={vagas} />
        </section>
      )}

      {mostrarLimpeza && (
        <Otimizar
          onFechar={() => setMostrarLimpeza(false)}
          onLimpou={async () => {
            const mem = await api.sondaMemoria();
            setRam(mem);
            void recarregar();
          }}
        />
      )}
    </>
  );
}
