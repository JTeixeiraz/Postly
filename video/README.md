# Vídeo de apresentação

Os 48 segundos que abrem o [site](https://jteixeiraz.github.io/Postly/), feitos
em [Remotion](https://remotion.dev) — React que renderiza vídeo, com narração e
trilha.

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
src/tokens.ts       cores, fontes e a conversão segundo → quadro
src/pecas.tsx       peças compartilhadas: marca, título, brilho, curva de saída
src/janela.tsx      o aplicativo desenhado: moldura, catálogo e grafo
src/integracoes.tsx as cenas de Claude Code e dos geradores de imagem
src/cenas.tsx       as nove cenas
src/Video.tsx       a montagem: duração, narração e trilha
public/audio/       trilha e as nove falas, numeradas na ordem das cenas
```

## Como a duração de cada cena é escolhida

Ela não é escolhida: é a duração da narração mais um respiro. Uma cena mais
curta que a fala corta a locução no meio; mais longa deixa a tela parada
esperando. O respiro extra vai onde a animação precisa de tempo próprio — a
trilha de revezamento acendendo, as barras do comparativo crescendo.

As durações ficam numa lista só, em `Video.tsx`: o início de cada cena é a soma
das anteriores, e calcular isso à mão é de onde vem o buraco de meio segundo
entre duas cenas.

## Os áudios não estão no repositório

`public/audio/` está no `.gitignore`. O vídeo renderizado usa a narração e a
trilha, e publicar um vídeo é uso normal — o que licença de banco de áudio
costuma proibir é redistribuir o arquivo de som isolado, que é o que
versioná-los aqui seria. Enquanto a licença da trilha não estiver documentada,
ela fica fora.

Para renderizar de novo, o diretório precisa ter:

```
public/audio/trilha.mp3          a trilha, sob a mixagem inteira
public/audio/narracao/01.mp3     abertura
                     /02.mp3     o problema
                     /03.mp3     o revezamento
                     /04.mp3     a inversão do hardware
                     /05.mp3     o tour de telas
                     /06.mp3     Claude Code
                     /07.mp3     geradores de imagem
                     /08.mp3     privacidade
                     /09.mp3     fecho
```

O texto de cada fala está em `src/Video.tsx`, ao lado da duração da cena.

## A trilha

A mesma de outros motions da casa. A escolha entre as duas candidatas foi
medida, não opinada: na faixa de 1 a 4 kHz, onde vive a inteligibilidade da
fala, esta fica 9 dB **abaixo** da narração e a outra ficava 15 dB **acima** —
ela mascararia a voz e exigiria ducking pesado. A 18% de volume ela é fundo, e
a voz sai 20 a 25 dB à frente.
