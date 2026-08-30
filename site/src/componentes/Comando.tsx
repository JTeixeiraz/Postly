import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useIdioma } from "../i18n";

const REPO = "JTeixeiraz/Postly";

/** O nome do sistema e o comando não são texto de página: "Linux" se escreve
 *  igual nos dois idiomas, e um comando traduzido deixa de colar no
 *  terminal. Só a nota abaixo dele muda. */
type Slug = "linux" | "macos" | "windows";

const SISTEMAS: { id: Slug; nome: string; comando: string }[] = [
  {
    id: "linux",
    nome: "Linux",
    comando: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/instalar.sh | bash`,
  },
  {
    id: "macos",
    nome: "macOS",
    comando: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/instalar.sh | bash`,
  },
  {
    // O mesmo script do Linux e do macOS: ele detecta o Windows pelo `uname`
    // do MSYS e baixa o instalador certo. O `winget install` fica de fora
    // enquanto o pacote está em revisão — anunciar um comando que devolve
    // "pacote não encontrado" é pior que não anunciar comando nenhum.
    id: "windows",
    nome: "Windows",
    comando: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/instalar.sh | bash`,
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
/** Os instaladores do Windows da versão mais recente.
 *
 *  Buscados na API em vez de escritos à mão: o nome do arquivo carrega a
 *  versão (`Postly_0.1.1_x64_en-US.msi`), e um link fixo apontaria para uma
 *  versão velha no dia seguinte ao próximo lançamento. Falhando a busca, os
 *  botões caem para a página de releases, que nunca some. */
const RELEASES = `https://github.com/${REPO}/releases/latest`;

function useInstaladoresWindows() {
  const [links, setLinks] = useState<{ msi?: string; exe?: string }>({});
  useEffect(() => {
    let vivo = true;
    fetch(`https://api.github.com/repos/${REPO}/releases/latest`)
      .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
      .then((d: { assets?: { name: string; browser_download_url: string }[] }) => {
        if (!vivo) return;
        const achar = (fim: string) =>
          d.assets?.find((a) => a.name.toLowerCase().endsWith(fim))?.browser_download_url;
        setLinks({ msi: achar(".msi"), exe: achar("-setup.exe") });
      })
      .catch(() => {});
    return () => {
      vivo = false;
    };
  }, []);
  return links;
}

export default function Comando({ compacto = false }: { compacto?: boolean }) {
  const { d } = useIdioma();
  const [sistema, setSistema] = useState(SISTEMAS[0]);
  const [copiado, setCopiado] = useState(false);
  const instaladores = useInstaladoresWindows();

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
      <div className="comando__abas" role="tablist" aria-label={d.comando.rotulo}>
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
              {copiado ? d.comando.copiado : d.comando.copiar}
            </motion.span>
          </AnimatePresence>
        </button>
      </div>

      <p className="comando__nota">{d.comando.notas[sistema.id]}</p>

      {/* Quem não tem o Git Bash não tem por que instalar um shell inteiro
          para rodar um instalador. Os dois formatos aparecem porque eles
          instalam diferente: o MSI vai para Programas e Recursos, o NSIS
          instala para o usuário e não pede elevação. */}
      {sistema.id === "windows" && !compacto && (
        <motion.div
          className="baixar"
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.2 }}
        >
          <span className="baixar__rotulo">{d.comando.ouBaixe}</span>
          <a className="baixar__op" href={instaladores.msi ?? RELEASES}>
            <strong>.msi</strong>
            <span>{d.comando.msi}</span>
          </a>
          <a className="baixar__op" href={instaladores.exe ?? RELEASES}>
            <strong>.exe</strong>
            <span>{d.comando.exe}</span>
          </a>
        </motion.div>
      )}
    </div>
  );
}
