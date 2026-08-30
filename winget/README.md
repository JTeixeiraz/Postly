# Publicação no Windows Package Manager

Os manifestos daqui são o que faz `winget install JTeixeiraz.Postly` funcionar.
Eles são submetidos ao [microsoft/winget-pkgs][pkgs] pelo
[workflow `winget`](../.github/workflows/winget.yml).

## O que você precisa fazer uma vez

O pull request vai para um repositório **de outra pessoa**, e o token que o
GitHub dá a um workflow só vale dentro do próprio repositório. Por isso é
preciso um token seu:

1. Crie um [Personal Access Token **clássico**][pat] com o escopo
   **`public_repo`** e nada mais — ele só precisa abrir PR em repositório
   público.
2. Guarde no repositório em **Settings → Secrets and variables → Actions**,
   com o nome exato **`WINGET_TOKEN`**.

Um token de escopo maior aqui seria um risco desnecessário: ele fica acessível
a qualquer workflow do repositório.

## Como publicar

Automático a cada release: assim que uma release é **publicada** (não em
rascunho), o workflow monta os manifestos, valida e abre o PR.

À mão, para testar sem publicar nada:

```
Actions → winget → Run workflow
  versão: 0.1.0     (ou vazio, para a release mais recente)
  ensaio: ✓          valida e guarda os manifestos como artefato, sem abrir PR
```

O ensaio vem marcado por padrão de propósito — abrir PR num repositório de
terceiro não deve ser efeito de apertar um botão por engano.

## O que acontece depois do PR

A Microsoft roda validação automática e revisão humana. O primeiro envio de um
pacote costuma demorar mais, e é comum receber pedidos de ajuste no manifesto.
Depois que a primeira versão entra, as seguintes passam quase sempre sem
intervenção.

## Por que rodar no Windows

`winget validate` só existe lá. Poderia-se submeter sem validar — a Microsoft
valida do lado dela de qualquer jeito — mas isso troca um erro pego em vinte
segundos por um ciclo de revisão perdido numa fila de milhares de PRs.

## Os arquivos

| Arquivo | Para quê |
|---|---|
| `JTeixeiraz.Postly.yaml` | Versão e locale padrão |
| `JTeixeiraz.Postly.installer.yaml` | URL, hash e tipo do instalador |
| `JTeixeiraz.Postly.locale.en-US.yaml` | O que `winget show` exibe |
| `JTeixeiraz.Postly.locale.pt-BR.yaml` | O mesmo, em português |

As âncoras `__VERSAO__`, `__URL__`, `__SHA256__` e `__DATA__` são preenchidas
pelo workflow. Não as substitua à mão: o hash precisa ser o do arquivo que a
release realmente publicou.

[pkgs]: https://github.com/microsoft/winget-pkgs
[pat]: https://github.com/settings/tokens/new?scopes=public_repo&description=Postly%20winget
