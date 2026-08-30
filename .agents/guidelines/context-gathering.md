# Reunir contexto

## Antes de escrever a primeira linha

1. **A constituição.** `@.agents/constitution.md` — as regras que não se negociam.
2. **A versão real das dependências.** `src-tauri/Cargo.toml` e `package.json`.
   Rust, Tauri e o ecossistema de modelos mudam rápido.
3. **O padrão que já existe no arquivo vizinho.** Este projeto tem convenções que
   não estão escritas em lugar nenhum além do próprio código: como um adaptador
   de imagem é estruturado, como uma tela lê preferências, como um erro viaja.

## Meça o que der para medir

Estimar quando dá para medir é o erro mais caro deste projeto.

```bash
# quanto tempo isso leva de verdade?
INICIO=$(date +%s); <comando>; echo "$(( $(date +%s) - INICIO ))s"

# esse endpoint existe? esse header é lido?
curl -s -o /dev/null -w "%{http_code}" -X POST "<url>" -H "<header>: valor-falso"

# esse binário aceita essa flag?
<binário> --help | grep -E "^\s+--flag"
```

**Um teste que discrimina vale mais que um que passa.** Ao verificar que algo
existe, verifique também que o método detecta a ausência: se `PowerToys` retorna
e `Postly` não, a busca funciona; se ambos retornassem vazio, o teste não prova nada.

## Quando a fonte é o binário

Strings de um executável são fonte primária quando a documentação é vaga:

```bash
strings -n 8 "$(readlink -f $(command -v claude))" | grep -oE "Usage limit[^\"]{0,60}" | sort -u
```

Foi assim que as marcas de detecção de cota saíram — do binário do CLI, não de
suposição sobre o formato da mensagem.

## Não confie no seu próprio medidor

Ferramenta de verificação também tem bug, e um falso positivo custa tempo:

- o medidor de contraste pelo DOM erra em fundo com alfa, canvas e gradiente —
  **meça os pixels**
- `pgrep -f <padrão>` casa com o próprio comando que o contém
- cliques sintéticos do `xdotool` não chegam no WebKitGTK sob XWayland

Quando o resultado surpreender, **duvide do medidor antes de duvidar do código**.
