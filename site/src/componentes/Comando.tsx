import { useState } from "react";
import { AnimatePresence, motion } from "motion/react";

const REPO = "JTeixeiraz/Postly";

interface Sistema {
  id: string;
  nome: string;
  comando: string;
  nota: string;
}

const SISTEMAS: Sistema[] = [
  {
    id: "linux",
    nome: "Linux",
    comando: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/instalar.sh | bash`,
    nota: "Instala um AppImage em ~/.local/bin. Roda em qualquer distro e não pede root.",
  },
  {
    id: "macos",
    nome: "macOS",
    comando: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/instalar.sh | bash`,
    nota: "O mesmo comando. Na primeira abertura, clique com o botão direito no app e escolha Abrir.",
  },
  {
    id: "windows",
    nome: "Windows",
    comando: `winget install --id JTeixeiraz.Postly`,
    nota: "Ainda não publicado no winget: por enquanto baixe o .msi na página de releases.",
  },
];

/** O comando de instalação, com o sistema escolhido pela pessoa.
 *
 *  Fica no alto da página e não no rodapé: quem chega já sabe o que quer, e
 *  esconder o comando atrás de um scroll inteiro é atrito sem motivo.
 *
 *  A cópia usa a API de clipboard com um caminho de reserva. Sem ele, todo
 *  visitante em contexto não seguro (um `file://`, uma prévia local) clicaria
 *  no botão e nada aconteceria, sem erro nenhum. */
export default function Comando({ compacto = false }: { compacto?: boolean }) {
  const [sistema, setSistema] = useState<Sistema>(SISTEMAS[0]);
  const [copiado, setCopiado] = useState(false);

  const copiar = async () => {
    try {
      await navigator.clipboard.writeText(sistema.comando);
    } catch {
      // Contexto não seguro: cai para o caminho antigo, que sempre funciona.
      const campo = document.createElement("textarea");
      campo.value = sistema.comando;
      campo.style.position = "fixed";
      campo.style.opacity = "0";
      document.body.appendChild(campo);
      campo.select();
      try {
        document.execCommand("copy");
      } finally {
        document.body.removeChild(campo);
      }
    }
    setCopiado(true);
    setTimeout(() => setCopiado(false), 1800);
  };

  return (
    <div className="comando" data-compacto={compacto}>
      <div className="comando__abas" role="tablist" aria-label="Sistema operacional">
        {SISTEMAS.map((s) => (
          <button
            key={s.id}
            role="tab"
            aria-selected={sistema.id === s.id}
            className="comando__aba"
            onClick={() => setSistema(s)}
          >
            {sistema.id === s.id && (
              // Uma camada compartilhada: ela desliza de uma aba para a outra
              // em vez de sumir aqui e aparecer ali.
              <motion.span
                className="comando__bolha"
                layoutId="aba-sistema"
                transition={{ type: "spring", stiffness: 420, damping: 34 }}
              />
            )}
            <span>{s.nome}</span>
          </button>
        ))}
      </div>

      <div className="comando__linha">
        <code className="comando__texto mono">
          <span className="comando__cifrao" aria-hidden>
            $
          </span>
          {sistema.comando}
        </code>
        <button className="comando__copiar" onClick={() => void copiar()}>
          <AnimatePresence mode="wait" initial={false}>
            <motion.span
              key={copiado ? "ok" : "copiar"}
              initial={{ opacity: 0, y: 5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -5 }}
              transition={{ duration: 0.16 }}
            >
              {copiado ? "copiado" : "copiar"}
            </motion.span>
          </AnimatePresence>
        </button>
      </div>

      <p className="comando__nota">{sistema.nota}</p>
    </div>
  );
}
