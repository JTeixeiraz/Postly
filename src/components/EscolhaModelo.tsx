import { useEffect, useState } from "react";
import { api } from "../api";
import { useIdioma } from "../i18n";
import type { Provedor, StatusProvedor, VagaClaude } from "../types";
import MarcaModelo from "./MarcaModelo";

/** Quem vai pensar este vídeo.
 *
 *  A ESCOLHA É POR VÍDEO, e não a preferência global. A decisão aqui é
 *  diferente da da campanha: rascunhar um corte com modelo local sai de graça e
 *  leva minutos; a versão que vai sair merece o CLI pago. Trocar a preferência
 *  global para isso mudaria a campanha junto, que ninguém pediu.
 *
 *  MOSTRA A CONSEQUÊNCIA, NÃO SÓ O NOME — a regra do produto. Cada cartão diz
 *  qual modelo assumiria o cargo que decide, se está instalado, e o que a
 *  escolha custa. "Ollama" e "Claude Code" lado a lado não ajudam ninguém a
 *  escolher; "Qwen3 14B, de graça, lento" e "Opus 5, rápido, custa da sua
 *  assinatura" ajudam. */
export default function EscolhaModelo({
  valor,
  aoEscolher,
}: {
  /** `null` = usar a preferência global. */
  valor: Provedor | null;
  aoEscolher: (p: Provedor | null) => void;
}) {
  const { d, f } = useIdioma();
  const [status, setStatus] = useState<StatusProvedor | null>(null);
  const [local, setLocal] = useState<string | null>(null);
  const [claude, setClaude] = useState<VagaClaude | null>(null);
  const [agy, setAgy] = useState<VagaClaude | null>(null);

  useEffect(() => {
    void api
      .statusProvedor()
      .then(setStatus)
      .catch(() => {});
    // O modelo que o cargo que decide receberia em cada caminho. É o mesmo
    // número que a aba Modelos mostra — duas telas dizendo coisas diferentes
    // sobre a mesma escolha seria pior que uma tela só.
    void api
      .elenco()
      .then((v) =>
        setLocal(v.find((x) => x.nivel === "alto")?.modelo_label ?? null),
      )
      .catch(() => {});
    void api
      .elencoClaude()
      .then((v) => setClaude(v.find((x) => x.nivel === "alto") ?? null))
      .catch(() => {});
    void api
      .elencoAntigravity()
      .then((v) => setAgy(v.find((x) => x.nivel === "alto") ?? null))
      .catch(() => {});
  }, []);

  if (!status) return <div className="skeleton" style={{ height: 96 }} />;

  const efetivo = valor ?? status.provedor;

  const opcoes: {
    id: Provedor;
    marca: string;
    titulo: string;
    modelo: string | null;
    nota: string;
    disponivel: boolean;
    ausente?: string;
  }[] = [
    {
      id: "ollama",
      marca: "Qwen",
      titulo: d.escolhaModelo.ollama,
      modelo: local,
      nota: d.escolhaModelo.ollamaNota,
      disponivel: true,
    },
    {
      id: "claude_code",
      marca: "Anthropic",
      titulo: d.escolhaModelo.claude,
      modelo: claude?.rotulo ?? null,
      nota: d.escolhaModelo.claudeNota,
      disponivel: status.claude_disponivel,
      ausente: d.escolhaModelo.claudeAusente,
    },
    {
      id: "antigravity",
      marca: "Google",
      titulo: d.escolhaModelo.agy,
      modelo: agy?.rotulo ?? null,
      nota: d.escolhaModelo.agyNota,
      disponivel: status.agy_disponivel,
      ausente: d.escolhaModelo.agyAusente,
    },
  ];

  return (
    <div className="field">
      <span>{d.escolhaModelo.titulo}</span>
      <div className="modelos-escolha">
        {opcoes.map((o) => (
          <button
            key={o.id}
            className="modelo-cartao"
            data-on={efetivo === o.id}
            data-ausente={!o.disponivel}
            aria-pressed={efetivo === o.id}
            // De propósito SEM `disabled`, pela mesma razão do seletor de
            // provedor: um botão desabilitado não responde ao clique, e quem
            // acabou de instalar o CLI ficaria batendo nele sem entender. O
            // clique também vale como "procura de novo".
            onClick={() => {
              aoEscolher(o.id);
              void api
                .statusProvedor()
                .then(setStatus)
                .catch(() => {});
            }}
          >
            <span className="modelo-cartao__marca">
              <MarcaModelo familia={o.marca} size={16} />
            </span>
            <span className="modelo-cartao__nome">{o.titulo}</span>
            {/* O modelo concreto é a informação que faz escolher. Sem ele os
                três cartões são só três nomes de programa. */}
            <span className="modelo-cartao__modelo">
              {o.disponivel ? (o.modelo ?? "—") : o.ausente}
            </span>
            <span className="modelo-cartao__nota">{o.nota}</span>
          </button>
        ))}
      </div>
      <span className="field__help">
        {valor === null
          ? f(d.escolhaModelo.herdado, { p: d.escolhaModelo[efetivo] })
          : d.escolhaModelo.soEsteVideo}
      </span>
    </div>
  );
}
