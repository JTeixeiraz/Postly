import type { ReactNode } from "react";
import { useIdioma } from "../i18n";

/** Explicacao longa fora do caminho.
 *
 *  As decisoes de arquitetura deste app sao incomuns o bastante para merecerem
 *  justificativa, mas quem vai rodar uma campanha nao precisa dela toda vez. O
 *  texto continua no produto, recolhido: abre quem quer entender, e nao ocupa a
 *  tela de quem quer trabalhar. */
export default function Porque({ children }: { children: ReactNode }) {
  const { d } = useIdioma();
  return (
    <details className="porque">
      <summary>{d.common.why}</summary>
      <div className="porque__corpo">{children}</div>
    </details>
  );
}
