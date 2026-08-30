# Constituição do Projeto

## Visão geral

Este documento reúne as decisões que governam o Postly. Tudo aqui é
**não negociável**: deve ser seguido a qualquer custo, salvo pedido explícito
de "bypass" do mantenedor.

## Propósito

O **Postly** é um aplicativo de desktop que roda um departamento de marketing na
máquina de quem usa. Quatro cargos de IA se revezam em modelos locais para
pesquisar, decidir a linha criativa, produzir a peça, auditar e publicar nas
redes sociais.

Projeto **open source, sem fins lucrativos**, licença MIT.

## A tese que não pode ser quebrada

> **Nunca há dois modelos residentes ao mesmo tempo.**

A cada troca de cargo o sistema mede a memória livre, escolhe o modelo mais forte
que couber naquele nível, sobe, recebe a resposta, grava a conversa, **descarrega
o modelo** e passa adiante só a mensagem que atravessa.

Isso não é otimização: é o que faz o produto caber numa máquina comum. Qualquer
mudança que mantenha dois modelos vivos ao mesmo tempo é uma emenda
constitucional, não um detalhe de implementação.

Corolário: **o nível do modelo é proporcional ao que o cargo entrega.** Quem
decide precisa raciocinar; quem cumpre briefing pronto, não. Isso vale em todos
os provedores — no Ollama por memória, no Claude Code por custo.

## Tecnologias mandatórias

Sair desta pilha exige emenda constitucional.

- **Casca desktop:** Tauri v2 (Rust + WebView)
- **Frontend:** React 18 + TypeScript, com `motion` para animação
- **Modelos locais:** Ollama
- **Navegador:** Playwright, em sidecar Node
- **Difusão local:** `sd-cli` do stable-diffusion.cpp
- **Serialização do grafo:** bincode + zstd

> **React 18, não 19.** O site importa componentes do aplicativo, e as duas
> versões não convivem. Subir um dos lados quebra o outro.

## Mandatos operacionais

- **O `Cargo.lock` é lei.** Todo comando de CI que resolve dependência usa
  `--locked`. Sem isso o runner compila versões diferentes das que passaram na
  máquina de quem escreveu — já aconteceu, e o sintoma apontou para o código errado.
- **Arquivos temporários** vão para o diretório de scratchpad da sessão, nunca
  para `/tmp` direto nem para a raiz do repositório.
- **Segredos nunca entram no repositório.** `vault.bin`, `vault.key`, `.env` e
  qualquer chave de API vivem no diretório de dados do sistema. O `.gitignore`
  recusa esses nomes como rede de segurança.
- **Nada que a pessoa não pediu é baixado.** Ollama, Chromium e o motor de
  difusão somam gigabytes. O provisionamento automático da primeira abertura é a
  exceção acordada; qualquer download novo precisa de um clique.

## Arquitetura e código

### Limites

- **Arquivos abaixo de 500 linhas.** O que passa é separado por momento de uso,
  não por conveniência de recorte.
- **Nenhuma dependência nova sem um motivo que caiba numa frase.** Numa máquina
  onde um modelo ocupa 20 GB, peso permanente por conveniência momentânea é
  troca ruim.

### Comentários

Comentário explica **por que**, nunca **o que**. O código já diz o que faz.

Comentário que envelheceu é pior que comentário ausente: quando o comportamento
mudar, o comentário muda junto ou sai.

### Fronteiras

- **Nenhum componente React chama `invoke` direto.** Tudo passa por `src/api.ts`,
  que é a única porta para o Rust.
- **A distinção entre material da marca e referência de terceiro é sagrada.**
  Material próprio pode aparecer na peça; referência de estilo vai só como texto.
  Misturar faz sair logotipo alheio na arte de quem usa.
- **Erro tipado, não texto.** Quando o chamador precisa distinguir causas, o erro
  vira enum. Reconhecer erro pelo texto da mensagem quebra na primeira tradução.

## Verificação

**Nenhuma entrega sem estes quatro passos, nesta ordem:**

```bash
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --test cerebro --test sistema --test limite
npm run build && npm run build --prefix site
```

- **`-D warnings` não se afrouxa.** Quando o clippy reclama, o código muda.
- **Os dois dicionários têm paridade garantida por tipo.** `en` é tipado como
  `Dicionario`; chave faltando não compila.
- **Testes que dependem de ambiente ficam `#[ignore]`** com o comando de execução
  no cabeçalho, e em binário próprio quando falam com serviço externo.

## Medir antes de afirmar

Este projeto tem histórico de defeitos que só apareceram quando alguém foi ver:

| O que se presumiu | O que era |
|---|---|
| o header do FLUX era `X-API-Key` | é `x-key`, e o erro é indistinguível de não enviar nada |
| o `claude` estaria no PATH | não está, num app aberto pelo ícone |
| o schema 1.12 do winget valia | o validador estável só conhece até 1.9 |
| o contraste calculado pelo DOM bastava | erra em alfa, canvas e gradiente |

**Número medido vence estimativa.** Quando a afirmação for sobre desempenho,
tamanho, tempo ou contraste, meça e cite o número.

## Interface

- **Nunca prometer o que não acontece.** Um texto que diz "pode fechar o app"
  quando fechar mata a campanha é pior que nenhum texto.
- **Botão que não faz nada é pior que a ausência dele.** Botão desabilitado não
  responde ao clique, e quem não entende por quê fica batendo nele.
- **Mostrar a consequência, não só o nome.** Um seletor de modo diz qual modelo
  cada opção traria e a que velocidade, não apenas "econômico" e "máximo".
- **Aviso depois da escolha, não antes.** Como ameaça prévia ele empurra para o
  meio sem argumento.

## Honestidade do produto

O README, o site e a interface dizem os limites em voz alta:

- o cofre **não** é um cofre de verdade contra um programa rodando com o mesmo usuário
- os seletores das redes sociais quebram, e vão continuar quebrando
- o TikTok espera vídeo e o sistema gera imagem
- sem GPU, um turno leva minutos ou dezenas deles

Um produto que só promete gera decepção; um que avisa gera usuário que sabe onde
pisa. **Remover um desses avisos é emenda constitucional.**
