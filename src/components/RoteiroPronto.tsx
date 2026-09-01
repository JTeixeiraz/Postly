import { useState } from "react";
import { useIdioma } from "../i18n";
import type { RoteiroDeLocucao } from "../types";

/** O roteiro de locução, com o link do ElevenLabs e a pasta de destino.
 *
 *  OS TRÊS ANDAM JUNTOS DE PROPÓSITO. A pessoa vai sair do app, colar o texto
 *  num site e voltar com um arquivo na mão. Faltando qualquer um dos três ela
 *  sai e não sabe voltar: sem o texto não tem o que colar, sem o link não sabe
 *  onde, e sem o caminho da pasta não sabe onde largar o áudio que gerou.
 *
 *  O caminho vai ABSOLUTO na tela. "Coloque na pasta de narração" é uma frase
 *  que não diz onde a pasta fica. */
export default function RoteiroPronto({ locucao }: { locucao: RoteiroDeLocucao }) {
  const { d, f } = useIdioma();
  const [copiado, setCopiado] = useState(false);

  const copiar = async () => {
    try {
      await navigator.clipboard.writeText(locucao.texto);
      setCopiado(true);
      setTimeout(() => setCopiado(false), 2000);
    } catch {
      // Área de transferência bloqueada: o texto continua na tela para copiar
      // à mão, então não vale um erro que interrompe.
    }
  };

  return (
    <section className="card">
      <div className="card__topo">
        <span className="card__titulo">{d.narracao.scriptTitle}</span>
        {/* Contagem medida pelo Postly, não a que o modelo afirmou: o prompt
            pede que ele diga, e ele erra. */}
        <span className="tag">
          {f(d.narracao.words, {
            n: locucao.palavras,
            s: locucao.segundos_estimados.toFixed(0),
          })}
        </span>
      </div>

      <p className="hint">{d.narracao.scriptLead}</p>

      {/* `despacho__texto` é a classe que o app já usa para texto de modelo em
          bloco. O roteiro é exatamente isso: saída de um cargo, para ler e
          copiar. */}
      <pre className="despacho__texto">{locucao.texto}</pre>

      <div className="row">
        <button className="btn btn--key" onClick={() => void copiar()}>
          {copiado ? d.narracao.copied : d.narracao.copy}
        </button>
        {/* Link normal e não um botão que abre: o alvo é um site de terceiro,
            e a pessoa merece ver para onde vai antes de clicar. */}
        <a className="btn" href={locucao.elevenlabs} target="_blank" rel="noreferrer noopener">
          {d.narracao.openEleven}
        </a>
      </div>

      <div className="modal__campo">
        <span className="read__k">{d.narracao.dropHere}</span>
        <code>{locucao.pasta}</code>
      </div>

      {/* O passo que fecha o ciclo. Sem esta frase a pessoa fica com um áudio
          gravado e nenhuma pista de que precisa rodar de novo. */}
      <div className="note" data-tone="warn">
        <span>{d.narracao.thenWhat}</span>
      </div>
    </section>
  );
}
