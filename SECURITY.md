# Política de segurança

## Reportar uma vulnerabilidade

**Não abra uma issue pública.** Use a [aba Security do
repositório](https://github.com/JTeixeiraz/Postly/security/advisories/new) para
reportar em privado — só você e os mantenedores enxergam.

Respondo em até 7 dias. Se a falha for confirmada, publico a correção e um
aviso creditando quem reportou, a menos que a pessoa prefira anonimato.

## O que é uma vulnerabilidade neste projeto

O Postly roda na máquina de quem usa e lida com credenciais de redes sociais e
com uma chave de API paga. Interessa especialmente:

- Vazamento da chave do Gemini, das credenciais ou do conteúdo do cofre para
  fora da máquina — em log, telemetria, transcrição ou requisição de rede.
- Execução de código arbitrário a partir de conteúdo que um modelo gera. Os
  cargos produzem texto que vira prompt do cargo seguinte, e nada disso deveria
  alcançar o shell.
- Escrita ou leitura de arquivo fora dos diretórios do aplicativo a partir de
  caminho vindo da interface.
- Qualquer coisa que faça o modo **Simular** publicar de verdade.

## O que NÃO é vulnerabilidade

- **O cofre não protege contra código rodando com o seu usuário.** O
  AES-256-GCM defende contra backup, sincronização de nuvem e alguém lendo o
  disco. Um programa com as suas permissões lê a chave — isso é limitação
  declarada, não falha. Ver o README.
- Seletor de rede social que parou de funcionar. Instagram e companhia mudam
  markup toda semana; isso é manutenção, e a issue pública é o lugar certo.
- Prompt que faz um modelo local escrever algo indesejado. Ele roda na sua
  máquina, com o seu contexto, e a saída passa por auditoria de outro cargo
  antes de virar publicação.

## Alerta conhecido, sem correção disponível

`RUSTSEC` reporta unsoundness nos `Iterator` de `glib::VariantStrIter` em
versões abaixo da 0.20. O projeto usa `glib` 0.18.5, e não por escolha: ele
entra por `tauri → muda → gtk → atk → glib`, e o Tauri 2.11.5 — a versão mais
recente — fixa `gtk` 0.18, que fixa `glib` 0.18. Não há como subir sem que o
Tauri suba primeiro.

O alerta fica aberto de propósito, para fechar sozinho quando a atualização
chegar. Nenhuma rota deste projeto chama `VariantStrIter`.

## Versões

O projeto está em 0.x. Correção de segurança sai na versão seguinte; não há
backport para versões anteriores.
