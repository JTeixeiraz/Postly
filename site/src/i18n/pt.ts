// O texto do site em português.
//
// Tudo que uma pessoa lê mora aqui, inclusive o que está dentro da vitrine:
// deixar a janela do aplicativo em português numa página em inglês faria a
// demonstração parecer uma captura de tela emprestada de outro produto.

export const pt = {
  meta: {
    lang: "pt-BR",
    titulo: "Postly — um departamento de marketing que roda na sua máquina",
    descricao:
      "Aplicativo desktop open source que orquestra modelos de IA locais como um time de marketing. Quatro cargos se revezam para criar, auditar e publicar nas suas redes. Um modelo por vez, sem mensalidade.",
    ogDescricao:
      "Um departamento de marketing que roda na sua máquina. Open source, sem fins lucrativos.",
  },

  nav: {
    como: "Como funciona",
    video: "Vídeo",
    telas: "Telas",
    claude: "Claude Code",
    instalacao: "Instalação",
    github: "GitHub",
    idioma: "Idioma",
  },

  heroi: {
    pilula: "open source, sem fins lucrativos",
    titulo1: "Um departamento de marketing",
    titulo2: "que roda na sua máquina.",
    linha:
      "Quatro cargos de IA se revezam em modelos locais para pesquisar o mercado, criar a peça, auditar e publicar nas suas redes. Um modelo por vez, porque numa máquina comum não cabem quatro.",
    verComo: "Ver como funciona",
    lerCodigo: "Ler o código",
  },

  faixa: "código aberto · sem fins lucrativos · roda na sua máquina",

  cargos: [
    { cargo: "Diretor Geral", nota: "Decide a estratégia entre as redes. Só existe quando há mais de uma." },
    { cargo: "Gerente de Setor", nota: "Um por rede. Lê o mercado pelo navegador e define a linha criativa." },
    { cargo: "Criador", nota: "Recebe briefing fechado e produz. Não decide nada, então não precisa raciocinar caro." },
    { cargo: "Auditor", nota: "Julga a peça pronta, com a imagem na frente. Reprovar devolve ao Criador." },
  ],
  trilhaRotulo: (postas: string) => `Percurso: ${postas}`,
  trilhaEntao: ", então ",

  video: {
    titulo: "Menos de um minuto",
    texto:
      "O projeto inteiro, do problema ao comando de instalação: o revezamento, a inversão do hardware, as integrações e o que nunca sai da sua máquina.",
    assistir: "Assistir à apresentação narrada, de 48 segundos",
    duracao: "48 s",
  },

  porque: {
    titulo: "Por que isto existe",
    texto:
      "Ferramentas de marketing com IA cobram por mês, guardam suas campanhas num servidor alheio e escondem qual modelo escreveu o quê.",
    p1: "O Postly faz o contrário: os modelos rodam no seu hardware, cada conversa fica em Markdown num arquivo seu, e o único custo variável é a geração de imagem.",
    p2: "A parte difícil nunca foi chamar um modelo. Foi fazer quatro caberem numa máquina comum, e é disso que trata o middleware por trás do revezamento.",
  },

  como: {
    titulo: "Um modelo por vez",
    texto:
      "Um agente de marketing precisa de raciocínio para decidir e de obediência para executar, e esses são modelos diferentes. Rodar todos ao mesmo tempo não cabe.",
    passos: [
      {
        titulo: "Mede a memória",
        texto: "Antes de cada turno o sistema lê quanta RAM está livre naquele instante, não quanta a máquina tem.",
      },
      {
        titulo: "Escolhe o mais forte que couber",
        texto: "Dentro do nível daquele cargo. Se nada couber, ele rebaixa o nível e avisa no relatório em vez de falhar.",
      },
      {
        titulo: "Grava a conversa inteira",
        texto: "System prompt, entrada, resposta completa e o raciocínio, num arquivo por turno. É o que permite auditar depois.",
      },
      {
        titulo: "Descarrega e passa adiante",
        texto: "Só a mensagem que atravessa segue para o próximo cargo. Nunca há dois modelos residentes ao mesmo tempo.",
      },
    ],
  },

  duelo: {
    titulo: "O catálogo ranqueia por velocidade, não por tamanho",
    texto:
      "A escolha muda com o hardware, e num PC sem GPU ela inverte a intuição de quem olha só o tamanho do arquivo.",
    denso: "denso · 14B ativos",
    moe: "MoE · 3B ativos de 30B",
    discoA: "9,3 GB em disco",
    discoB: "19 GB em disco",
    nota:
      "Medido nesta máquina de desenvolvimento: Ryzen 5000, sem GPU utilizável. O modelo que ocupa o dobro de memória gera quase dez vezes mais rápido, porque só os especialistas ativos passam pela CPU. Otimizar por tamanho de arquivo leva à escolha errada.",
  },

  telas: {
    titulo: "O que você abre",
    texto:
      "O aplicativo, rodando aqui na página. Mesmo React, mesmo CSS — troque de aba, arraste os nodes do grafo.",
    legendas: {
      modelos: "O catálogo ranqueia por velocidade medida, não por tamanho de arquivo. Baixar e remover acontecem daqui.",
      campanha: "A trilha mostra onde o despacho está. Cada turno leva minutos, então dá para sair e voltar.",
      cerebro: "Arraste os nodes. A vizinhança à direita é exatamente o que um agente recebe ao consultar.",
    },
    avancar: "Avançar o turno",
  },

  laco: {
    titulo: "O desempenho volta para dentro",
    texto:
      "Um gerador de conteúdo sem retorno de desempenho repete o que o modelo acha bonito, não o que funcionou.",
    p1: "Você registra o resultado de cada publicação e o sistema ranqueia as peças contra a mediana da própria conta. Essa leitura entra no prompt do cargo que decide a próxima campanha, com o número exato a bater.",
    p2: "A regra é sempre superar e nunca repetir: o que rendeu vira piso, não molde. A única exceção é o acerto extraordinário, quando uma peça bate três vezes a mediana — aí deixou de ser sorte, e vale seguir naquela linha enquanto ela render.",
  },

  privacidade: {
    titulo: "O que sai da sua máquina",
    sai: "sai",
    fica: "fica",
    saiItens: [
      "O prompt da imagem, para o serviço de arte que você escolheu.",
      "O tráfego do navegador para as redes onde você publica, na sua sessão.",
    ],
    ficaItens: [
      "Os modelos e tudo que eles escrevem.",
      "O grafo de contexto, as campanhas e as transcrições.",
      "As credenciais, cifradas em AES-256-GCM.",
    ],
    nota:
      "Não há telemetria, não há servidor do projeto, não há conta para criar. Sobre o cofre, uma ressalva que costuma ser vendida com mais confiança do que merece: ele protege contra backup, sincronização de nuvem e alguém lendo o disco. Não protege contra um programa rodando com o seu usuário.",
  },

  claude: {
    pilula: "integração opcional",
    titulo: "Ou deixe o Claude Code executar os turnos",
    texto:
      "Modelo local numa CPU entrega perto de 1 token por segundo, e uma campanha inteira leva a tarde. Quem já assina o Claude Code pode apontar os quatro cargos para ele e a mesma campanha termina em minutos.",
    itens: [
      {
        titulo: "O nível do cargo continua mandando",
        texto:
          "Só muda o eixo: quem decide e julga vai para o Opus, quem audita dentro de critério recebido vai para o Sonnet, e quem cumpre briefing pronto vai para o Haiku. O revezamento é o mesmo.",
      },
      {
        titulo: "Pela sua assinatura, não por token",
        texto:
          "O Postly executa o binário `claude` que já está na sua máquina, com a sessão que você já abriu. Não existe campo de chave de API em lugar nenhum do aplicativo, e o cofre nunca guardou uma.",
      },
      {
        titulo: "As ferramentas ficam desligadas",
        texto:
          "Bash, leitura e escrita de arquivo, busca na web: nada disso é oferecido ao cargo. Ele escreve texto e devolve. Um agente de marketing não tem por que mexer no seu disco.",
      },
    ],
    nota:
      "Uma armadilha que vale contar: o CLI prefere ANTHROPIC_API_KEY quando ela existe no ambiente, e um processo filho herda o ambiente do pai. Quem tivesse essa variável exportada no shell passaria a pagar por token sem perceber. O Postly a remove do processo filho, junto das credenciais de Bedrock e Vertex. O turno roda pela assinatura, ou não roda.",
    ressalva:
      "Claude e Claude Code são marcas da Anthropic. O Postly é um projeto independente, sem vínculo com a Anthropic.",
  },

  instalacao: {
    titulo: "Instalação",
    texto:
      "Um comando. Ele detecta o sistema, confirma com você e baixa o pacote da versão mais recente.",
    requisitos: [
      {
        titulo: "Ollama",
        texto:
          "Você não precisa instalar antes. O app faz isso na primeira abertura, junto do navegador que publica, com barra de progresso.",
      },
      {
        titulo: "Uma chave de imagem",
        texto:
          "Gemini, OpenAI, FLUX, Stability AI ou Higgsfield. Só a arte precisa de serviço externo; o texto todo sai dos modelos locais.",
      },
      {
        titulo: "Memória",
        texto:
          "16 GB para um uso confortável. Roda em 8 GB com modelos menores, e o catálogo mostra quais cabem antes de você baixar.",
      },
    ],
  },

  comando: {
    rotulo: "Sistema operacional",
    maisOpcoes: "Ou baixe o instalador, sem Git Bash →",
    ouBaixe: "Ou baixe o instalador:",
    emBreve: "em revisão",
    notaWinget: "O pacote está na fila do winget-pkgs. Este comando passa a funcionar assim que ele for aprovado.",
    msi: "instala para todos, pede administrador",
    exe: "instala só para você, sem administrador",
    copiar: "copiar",
    copiado: "copiado",
    notas: {
      linux: "Instala um AppImage em ~/.local/bin. Roda em qualquer distro e não pede root.",
      macos: "O mesmo comando. Na primeira abertura, clique com o botão direito no app e escolha Abrir.",
      windows: "Cole no Git Bash, que vem junto com o Git para Windows. O script detecta o sistema e baixa o instalador certo.",
    },
  },

  aberto: {
    titulo: "Sem fins lucrativos",
    p1: "O Postly é gratuito e licenciado sob MIT. Não há versão paga, plano, assinatura, nem intenção de criar uma. O código inteiro está no GitHub, incluindo o do cofre que guarda suas credenciais, porque um app que pede chave de API precisa ser auditável.",
    p2: "Se algo aqui foi útil, o retorno que faz diferença é uma issue com um problema reproduzível ou um adaptador de rede social que parou de funcionar.",
    ver: "Ver no GitHub",
    issue: "Abrir uma issue",
  },

  rodape: { nota: "MIT · feito por" },

  vitrine: {
    abas: { modelos: "Modelos", campanha: "Campanha", cerebro: "Cérebro" },
    ram: "21,5 GB livre",

    modelos: {
      titulo: "O que roda aqui",
      texto: "Você não escolhe. A cada cargo, sobe o modelo mais forte que couber.",
      teto: "teto por modelo",
      tetoNota: "CPU apenas",
      cabe: "cabe",
      cabeNota: "1 fora do alcance deste hardware",
      baixado: "baixado",
      todas: "Todas",
      avancado: "Configuração avançada",
      decisao: "Decisão",
      decisaoNota: "Diretor e Gerente. Escolhem a linguagem e julgam a peça.",
      tagBaixado: "baixado",
      tagNaoCabe: "não cabe agora",
      tagVisao: "enxerga imagem",
      remover: "Remover",
      baixar: "Baixar",
      denso: (p: string) => `denso · ${p}B ativos`,
      moe: (a: string, p: number) => `MoE · ${a}B ativos de ${p}B`,
      notas: {
        "qwen3:30b-a3b": "Só os especialistas ativos passam pela CPU — por isso é o mais rápido aqui.",
        "gpt-oss:20b": "Raciocínio forte com pegada modesta. Bom para o cargo que decide.",
        "llama3.3:70b": "Denso e caro. Só faz sentido com muita VRAM.",
        "gemma3:4b": "Cumpre briefing pronto sem inventar. Enxerga imagem.",
        "deepseek-r1:14b": "Pensa antes de responder. Serve bem ao auditor.",
        "mistral-small:24b": "Cabe no disco, mas não na memória livre agora.",
        "phi-4:14b": "Pequeno e obediente. Bom custo por token.",
        "granite3.3:8b": "Licença permissiva e saída previsível.",
      } as Record<string, string>,
      razoes: {
        "llama3.3:70b": "Não cabe: precisa de 45,2 GB e há 21,5 GB livres.",
      } as Record<string, string>,
    },

    campanha: {
      titulo: "Campanha",
      texto: "Escreva o que quer atingir. O resto é o revezamento.",
      trilha: "Trilha de revezamento",
      concluida: "concluída",
      turno: (i: number, n: number) => `turno ${i} de ${n}`,
      objetivo: "Objetivo",
      objetivoRotulo: "O que você quer atingir",
      objetivoTexto:
        "Apresentar o torrado novo para quem já compra café em grão e reclama de acidez.",
      redes: "Redes",
      instagramNota: "imagem quadrada, legenda curta",
      linkedinNota: "texto longo, corte em 210 caracteres",
      previsao: "O que vai acontecer",
      turnos: "turnos de agente",
      imagens: "imagens",
      imagensNota: "pelo Gemini",
      tempo: "tempo estimado",
      min: "min",
      rodar: "Rodar campanha",
      rodape:
        "Estimado pela velocidade medida nesta máquina. O padrão é Simular: monta a publicação inteira e para antes do último clique.",
      postas: [
        { cargo: "Diretor Geral", nota: "decide a linha" },
        { cargo: "Gerente de Setor", nota: "Instagram" },
        { cargo: "Criador de Conteúdo", nota: "executa o briefing" },
        { cargo: "Auditor", nota: "julga a peça" },
      ],
    },

    cerebro: {
      titulo: "Cérebro",
      texto: "O contexto que todos os cargos compartilham, em grafo ponderado.",
      dica: "Arraste um node para fixá-lo. Roda do mouse aproxima; duplo clique reenquadra.",
      nenhum: "nenhum node",
      vizinhanca: "vizinhança ordenada",
      explicacao:
        "É exatamente isto que um agente recebe ao consultar: já ordenado por peso e cortado por limiar, para o corte não acontecer dentro do modelo.",
      nodes: {
        publico_alvo: "publico_alvo", tom_de_voz: "tom_de_voz", instagram: "instagram",
        linkedin: "linkedin", produto: "produto", concorrente: "concorrente",
        prova_social: "prova_social", objecao_preco: "objecao_preco",
        sazonalidade: "sazonalidade", formato_carrossel: "formato_carrossel",
        identidade_visual: "identidade_visual", campanha_agosto: "campanha_agosto",
      } as Record<string, string>,
      colNode: "node",
      colRelacao: "relação",
      colPeso: "peso",
      relacoes: {
        define: "define", vive_em: "vive_em", levanta: "levanta", combina: "combina",
        sustenta: "sustenta", compete: "compete", prefere: "prefere", rodou_em: "rodou_em",
        considera: "considera", responde: "responde", restringe: "restringe",
        explora: "explora", ajusta: "ajusta",
      } as Record<string, string>,
    },
  },
};

// Sem `as const`: com ele cada frase vira o seu próprio tipo literal, e o
// dicionário em inglês deixa de ser atribuível ao tipo — "How it works" não é
// do tipo "Como funciona". O que precisa ser exato aqui é a FORMA, não o
// texto, e é isso que `typeof pt` sem const congela.
export type Dicionario = typeof pt;
