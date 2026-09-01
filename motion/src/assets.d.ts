/** O bundler emite o arquivo e devolve a URL dele.
 *
 *  Sem esta declaração o TypeScript não sabe o que um `import` de fonte
 *  devolve, e o build do render quebraria numa checagem de tipo por causa de um
 *  arquivo binário. */
declare module "*.woff2" {
  const url: string;
  export default url;
}
