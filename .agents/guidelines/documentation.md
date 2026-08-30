# Documentação

## Onde cada coisa mora

| O quê | Onde | Por quê |
|---|---|---|
| **Por que o código é assim** | comentário no próprio arquivo | é onde quem vai mexer está olhando |
| **O que mudou e por quê** | mensagem do commit | é o registro da decisão no tempo |
| **Como o produto funciona** | ``$HOME/Documentos/Obsidian Vault`/Postly/` | é a memória de longo prazo do mantenedor |
| **O que quem chega precisa saber** | `README.md` e o site | é a porta de entrada |
| **Limitação sem correção** | `SECURITY.md` ou a seção de limites | é onde quem for atingido vai ler |

## A mensagem de commit

Diz **o que mudou e por que**, não o que o diff já mostra. O padrão aqui é
parágrafo, não bullet: as decisões têm razão, e razão não cabe em tópico.

Quando um número decidiu a escolha, o número entra:

> `rand 0.8.8` no lock, `0.10.2` no runner — e no log do CI a 0.8 sequer aparece.

Quando um teste prova a regra, o teste entra:

> há teste que falha se os três modos passarem a escolher o mesmo, porque aí o
> seletor viraria enfeite.

**Um assunto por commit.** Misturar site com aplicativo obriga quem lê o histórico
a separar de novo.

## O vault do Obsidian

Estrutura: `Postly.md` (MOC) na raiz, `Postly/NN - Título.md` ao lado.

Cada nota tem frontmatter com `tags` e `data_criacao`, abre com
`> [!info] Parte de [[Postly]]`, e usa callouts para separar o que é decisão do
que é descrição.

Vale registrar:

- **causa raiz, não sintoma** — "o CI resolveu `rand` 0.10 porque não usava
  `--locked`", e não "o build quebrou"
- **falsos positivos** — saber que o medidor de contraste erra em alfa vale
  tanto quanto saber que a cor passa
- **o que foi medido**, com o número

Não vale registrar o que o `git log` já conta melhor.

## Quando atualizar

No mesmo turno em que a mudança acontece, não "depois". Documentação que espera
um momento melhor não chega.

Ao terminar, confira que os links resolvem:

```bash
cd "$HOME/Documentos/Obsidian Vault" && python3 -c "
import pathlib, re
ex={p.stem for p in pathlib.Path('.').rglob('*.md') if '.obsidian' not in str(p)}
q=[n for f in pathlib.Path('.').rglob('*.md') if '.obsidian' not in str(f)
     for n in re.findall(r'\[\[([^\]|#]+)', f.read_text()) if n.strip() not in ex]
print('links quebrados:', sorted(set(q)) or 'nenhum')"
```
