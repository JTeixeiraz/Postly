# Vídeo de apresentação

Os 40 segundos que abrem o [site](https://jteixeiraz.github.io/Postly/), feitos
em [Remotion](https://remotion.dev) — React que renderiza vídeo.

```bash
npm install
npm run dev      # estúdio, com pré-visualização quadro a quadro
npm run render   # grava em ../site/public/postly.mp4
```

## Por que não há captura de tela aqui

As telas do aplicativo são **desenhadas** em `src/janela.tsx`, com os mesmos
tokens do produto (`src/tokens.ts`, espelho de `src/styles.css` do app). Um PNG
de tela mostra a interface parada; o que vale mostrar é ela trabalhando — a
lista do catálogo chegando linha a linha, o grafo se abrindo do centro para as
relações mais fortes.

E há uma razão técnica: a renderização é determinística, quadro a quadro. Uma
transição CSS ou uma animação de biblioteca não existe no arquivo final —
**toda animação aqui é função do frame atual**, via `interpolate` e a curva de
saída em `src/pecas.tsx`.

## Estrutura

```
src/tokens.ts    cores, fontes e a conversão segundo → quadro
src/pecas.tsx    peças compartilhadas: marca, título, brilho, curva de saída
src/janela.tsx   o aplicativo desenhado: moldura, catálogo e grafo
src/cenas.tsx    as sete cenas
src/Video.tsx    a montagem, com a duração de cada cena
```

As durações ficam numa lista só, em `Video.tsx`: o início de cada cena é a soma
das anteriores, e calcular isso à mão é de onde vem o buraco de meio segundo
entre duas cenas.
