# Trade-offs

## Quando a informação falta

Faça tudo o que **não** depende da resposta primeiro. Para o que depende, escolha
uma das duas:

- **assuma e diga a suposição em voz alta**, quando errar é barato de corrigir
- **pergunte**, quando seguir sob qualquer suposição tornaria o trabalho inútil
  se estivesse errada

Não pergunte o que dá para medir.

## Quando duas coisas boas se contradizem

Escreva a razão da escolha no código, não na mensagem do commit — o commit some
do campo de visão de quem lê o arquivo seis meses depois.

```rust
// Copia, e nao aponta para o original: um caminho para a pasta de Downloads
// quebra no dia em que ela limpar o disco, e a campanha falharia meses depois
// sem explicacao.
```

## Peso permanente por conveniência momentânea

A pergunta é sempre: **isso fica no binário de todo mundo, para resolver o quê?**

- descompactar com o `unzip` do sistema em vez de trazer um crate de zip para
  uma operação que acontece uma vez na vida do app — **sim**
- 150 KB de WebGL por um fundo decorativo — **não**, refeito em canvas 2D
- a feature `test` do Tauri em `dev-dependencies`, que não vai para a release —
  **sim**, e destrava testar um caminho que só se veria estourando cota de propósito

## Dívida que se aceita, e como

Quando não dá para resolver agora, o registro vai onde quem for atingido vai ler:

- alerta de segurança sem correção alcançável → `SECURITY.md`, com o motivo
  (o `glib` só sobe quando o Tauri migrar de gtk-rs)
- limitação de produto → README e site, na seção de limites
- decisão que pode ser revisitada → o comentário no código, com o gatilho
  ("se souber a licença da trilha, dá para versionar")

**Nunca em comentário `TODO` solto.** Ele não é lido por quem precisa.

## Quando o usuário corrige uma escolha sua

Aceite e execute. Se houver um detalhe técnico que ele não podia saber, diga em
uma frase e siga — não discuta a decisão dele.

O contorno branco do adesivo parecia resíduo de recorte; era a estética. A
correção veio do mantenedor, e a implementação mudou.
