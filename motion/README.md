# motion — a biblioteca de cenas

O Motion Designer **não escreve TSX**. Ele devolve um JSON de cenas, e o que
está aqui renderiza. As razões estão em `src-tauri/src/video/spec.rs`, no topo
do arquivo; a curta é que o provedor padrão do Postly é um modelo local pequeno
e ele não escreve código que compila.

```
roteiro.json  ──▶  Video.tsx  ──▶  cenas/*.tsx  ──▶  .mp4
   (o cargo)        (montagem)      (o vocabulário)     (o sidecar)
```

## O contrato

Um tipo de cena novo precisa de três coisas, e as três têm teste:

1. uma variante em `TipoCena` (`src-tauri/src/video/spec.rs`)
2. uma linha em `descrever()` (`src-tauri/src/video/prompts.rs`) — o catálogo do
   prompt é **gerado do enum**, e há teste que falha se o tipo novo não tiver
   descrição
3. um ramo em `Cena.tsx` aqui

## Por que não reusa `video/`

`video/` é a apresentação de 48s do produto: nove cenas fixas, escritas à mão,
com narração gravada. Aqui as cenas são montadas em tempo de execução a partir
de assets que o Postly nunca viu. São dois problemas diferentes com a mesma
biblioteca — juntar faria a apresentação depender de um JSON que ela não usa.

Os **tokens** são compartilhados de propósito (`tokens.ts` é cópia do de
`video/`): três superfícies com a mesma marca já é o padrão do projeto.

## React 19 aqui, React 18 no app

A constituição fixa React 18 porque **o site importa componentes do aplicativo**
e as duas versões não convivem. Isso não alcança este projeto: `motion/` tem
`package.json` próprio, roda dentro do sidecar Node e nunca importa nada de
`src/`. É a mesma situação de `video/`, que já usa 19.2.3 desde a primeira
versão.

## Rodar à mão

O render normal acontece pelo sidecar, chamado pelo app. Para conferir uma cena
sem passar pelo app, o Remotion Studio lê um roteiro de exemplo:

```bash
npm ci --prefix motion
npx remotion studio motion/src/Root.tsx
```
