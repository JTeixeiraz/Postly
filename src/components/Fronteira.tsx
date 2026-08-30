import { Component, type ErrorInfo, type ReactNode } from "react";
import { IconAlert } from "./Icons";

/** Fronteira de erro.
 *
 *  Este app lê máquinas que ninguém previu: driver ausente, comando que devolve
 *  formato inesperado, campo que não veio. Sem esta fronteira, qualquer um
 *  desses vira tela preta sem explicação, que é o pior resultado possível para
 *  quem baixou o projeto e está tentando entender por que não abriu. */
export default class Fronteira extends Component<
  { children: ReactNode },
  { erro: Error | null; pilha: string }
> {
  state = { erro: null as Error | null, pilha: "" };

  static getDerivedStateFromError(erro: Error) {
    return { erro };
  }

  componentDidCatch(erro: Error, info: ErrorInfo) {
    this.setState({ pilha: info.componentStack ?? "" });
    console.error("[postly] falha de renderização", erro, info);
  }

  render() {
    if (!this.state.erro) return this.props.children;

    return (
      <div className="page" style={{ paddingTop: 64 }}>
        <div className="empty">
          <IconAlert size={26} />
          <h3>A interface parou de desenhar.</h3>
          <p className="hint">
            O erro está abaixo. Ele quase sempre vem de um dado que a máquina devolveu num
            formato que o app não esperava — vale abrir uma issue com este texto.
          </p>
          <pre className="raw" style={{ maxWidth: "100%" }}>
            {this.state.erro.message}
            {this.state.pilha}
          </pre>
          <button className="btn btn--key" onClick={() => window.location.reload()}>
            Recarregar
          </button>
        </div>
      </div>
    );
  }
}
