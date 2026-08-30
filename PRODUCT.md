# Postly

Um departamento de marketing que roda na máquina de quem usa.

## O que é

Aplicativo desktop que orquestra modelos de IA locais (Ollama) como um time de
marketing. Quatro cargos se revezam para pesquisar o mercado, decidir a linha
criativa, produzir a peça, auditar e publicar nas redes sociais — com **um único
modelo residente por vez**, porque numa máquina comum não cabem quatro.

## Para quem

Quem faz o próprio marketing e não quer mensalidade nem entregar as campanhas
para um servidor alheio: dono de negócio pequeno, freelancer, criador. Também
serve a quem estuda orquestração de agentes: o pipeline inteiro fica gravado em
Markdown, turno por turno.

## O que o diferencia

1. **Middleware, não chat com ferramentas.** A cada troca de cargo o sistema
   mede a memória livre, escolhe o modelo mais forte que couber naquele nível,
   sobe, grava a conversa, descarrega e repassa só a mensagem que atravessa.
2. **O catálogo ranqueia por vazão, não por tamanho.** Sem GPU, um MoE de 30B
   com 3B ativos gera mais rápido que um denso de 14B. Otimizar por tamanho de
   arquivo leva à escolha errada.
3. **O laço fecha.** O desempenho medido das publicações entra no prompt da
   campanha seguinte, com a regra de sempre superar e nunca repetir.
4. **Nada sai da máquina** exceto a geração de imagem e o navegador que publica
   nas contas de quem usa.

## Register

brand — o site é a face pública de um projeto open source sem fins lucrativos.
O app em si é `product`.

## Sistema visual (já comprometido, não reabrir)

O app roda desde a primeira versão em carvão + lime, com Geist e Geist Mono. O
site herda os mesmos tokens: um visitante que instala precisa reconhecer o que
abriu.

| Papel | Token | Valor |
|---|---|---|
| Fundo | `--bg` | `#15181C` |
| Cartão | `--card` | `#1F2329` |
| Ação e "acontecendo agora" | `--act` | `#C9F227` |
| Tinta | `--ink` | `#F7F8F9` |
| Tinta secundária | `--ink-2` | `#AEB4BD` |
| Linha | `--line` | `#2F3540` |

Estratégia de cor: **committed**. O lime aparece pouco e por isso é visto; ele
significa exatamente uma coisa, aqui está quente agora.

## Lane estética do site

**A trilha de revezamento como espinha da página.** Uma linha atravessa o site
inteiro e cada seção é uma posta; a linha se acende conforme a pessoa rola. Sai
da arquitetura real do produto — um despacho passando de posta em posta — e não
de um template de landing.

Não é editorial-tipográfico, não é Stripe-minimal, não é acid-maximalism. O
movimento existe porque o produto É um percurso; numa página sobre outra coisa
esta escolha seria decoração.

## Imagens

Capturas reais das oito telas do app, em `docs/capturas/`. Nada de mockup
genérico: o que o visitante vê na página é o que ele abre depois de instalar.
