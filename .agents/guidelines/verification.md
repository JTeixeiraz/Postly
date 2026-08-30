# Verificar

## Os quatro passos, sempre

```bash
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --test cerebro --test sistema --test limite
npm run build && npm run build --prefix site
```

Mais, quando a mudança tocar a janela:

```bash
cargo build --manifest-path src-tauri/Cargo.toml
GDK_BACKEND=wayland timeout 12 ./src-tauri/target/debug/postly   # sem saída = sobe limpo
```

## O que uma captura de tela prova, e o que não prova

Prova que o desenho está certo. **Não prova** que o caminho até ele funciona.

Três camadas, e cada uma precisa da sua própria evidência:

| Camada | Como provar |
|---|---|
| a lógica pura | teste de unidade |
| o caminho no Rust | teste de integração com `AppHandle` de teste (feature `test` do Tauri) |
| a costura até a tela | evento emitido do Rust, chegando no `listen` do componente |

Para a terceira, o clique de volta é disparado **por dentro** da janela via
`eval` — cliques sintéticos do XTEST não chegam no WebKitGTK sob XWayland.

## Escreva o teste que falha pelo motivo certo

Um teste que passa não diz nada; um que **falharia** se a regra fosse quebrada
diz tudo.

- `encerrar_nao_espera_nada` cronometra e falha se levar mais de 5 s com uma cota
  que só volta em 1 h. Se o encerrar passasse a respeitar o relógio, o teste
  levaria uma hora — e diria por quê.
- `os_tres_modos_nao_escolhem_a_mesma_coisa` falha se os modos convergirem, porque
  aí o seletor viraria enfeite.
- `o_slug_nao_deixa_escapar_da_galeria` prova que `../../etc` não escreve fora.

## Quando um teste depende do ambiente

`#[ignore]`, com o comando de execução no cabeçalho:

```rust
/// cargo test --test limite -- --ignored --exact reconhece_a_saida_real_do_cli
#[tokio::test]
#[ignore]
```

E em **binário próprio** quando fala com serviço externo, para não entrar na
lista de alvos do CI.

## Antes de dizer "está pronto"

- rode os quatro passos e **leia a saída**, não o código de retorno
- se algo falhou, diga o que falhou com a saída junto
- se algo foi pulado, diga que foi pulado
