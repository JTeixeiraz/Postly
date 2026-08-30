#!/usr/bin/env bash
#
# Instalador do Postly.
#
#   curl -fsSL https://raw.githubusercontent.com/JTeixeiraz/Postly/main/scripts/instalar.sh | bash
#
# Detecta o sistema, confirma com voce, e instala a versao pronta da release
# mais recente. Se nao houver pacote para a sua combinacao de sistema e
# arquitetura, oferece compilar do codigo-fonte.

set -euo pipefail

# ── a unica linha que precisa mudar ao publicar o repositorio ───────────────
REPO="${POSTLY_REPO:-JTeixeiraz/Postly}"

VERDE=$'\033[38;5;155m'
CINZA=$'\033[38;5;245m'
FORTE=$'\033[1m'
ZERA=$'\033[0m'

msg()  { printf '%s\n' "$*"; }
erro() { printf '\033[38;5;203m%s\033[0m\n' "$*" >&2; }
tem()  { command -v "$1" >/dev/null 2>&1; }

# ── menu com setas ──────────────────────────────────────────────────────────
#
# Guarda o indice escolhido em ESCOLHA. Volta ao primeiro item quando o
# terminal nao e interativo (rodando por pipe, por exemplo), porque ali nao ha
# teclado para ler e travar esperando seria pior que assumir o padrao.
ESCOLHA=0
menu() {
  local titulo="$1"; shift
  local opcoes=("$@")
  local sel="${PRE_SELECIONADO:-0}"
  local n=${#opcoes[@]}

  if [ ! -t 0 ] || [ ! -t 1 ]; then
    ESCOLHA="$sel"
    msg "${CINZA}${titulo}: ${opcoes[$sel]} (detectado)${ZERA}"
    return
  fi

  printf '%s\n' "$titulo"
  printf '%s\n' "${CINZA}setas para escolher, Enter para confirmar${ZERA}"

  local primeira=1
  while true; do
    [ "$primeira" -eq 0 ] && printf '\033[%dA' "$n"
    primeira=0
    local i
    for i in "${!opcoes[@]}"; do
      printf '\033[2K'
      if [ "$i" -eq "$sel" ]; then
        printf '  %s❯ %s%s\n' "$VERDE$FORTE" "${opcoes[$i]}" "$ZERA"
      else
        printf '    %s%s\n' "${opcoes[$i]}" "$ZERA"
      fi
    done

    local tecla resto
    IFS= read -rsn1 tecla </dev/tty || { ESCOLHA="$sel"; return; }
    case "$tecla" in
      $'\x1b')
        read -rsn2 -t 0.05 resto </dev/tty || resto=""
        case "$resto" in
          '[A') [ "$sel" -gt 0 ] && sel=$((sel - 1)) ;;
          '[B') [ "$sel" -lt $((n - 1)) ] && sel=$((sel + 1)) ;;
        esac
        ;;
      'k') [ "$sel" -gt 0 ] && sel=$((sel - 1)) ;;
      'j') [ "$sel" -lt $((n - 1)) ] && sel=$((sel + 1)) ;;
      '') ESCOLHA="$sel"; return ;;
      'q') erro "Cancelado."; exit 1 ;;
    esac
  done
}

# ── deteccao ────────────────────────────────────────────────────────────────
detectar_so() {
  case "$(uname -s)" in
    Linux)  echo linux ;;
    Darwin) echo macos ;;
    MINGW*|MSYS*|CYGWIN*) echo windows ;;
    *) echo desconhecido ;;
  esac
}

detectar_arq() {
  case "$(uname -m)" in
    x86_64|amd64) echo x64 ;;
    arm64|aarch64) echo arm64 ;;
    *) echo desconhecido ;;
  esac
}

# ── instalacao por sistema ──────────────────────────────────────────────────
baixar() {
  local url="$1" destino="$2"
  msg "${CINZA}baixando...${ZERA}"
  if tem curl; then
    curl -fL --progress-bar "$url" -o "$destino"
  elif tem wget; then
    wget -q --show-progress "$url" -O "$destino"
  else
    erro "Preciso de curl ou wget para baixar."; exit 1
  fi
}

# Pergunta a API do GitHub qual e o arquivo certo para esta combinacao.
# Sem `jq` como dependencia: o campo que interessa e uma linha so do JSON.
achar_ativo() {
  local padrao="$1"
  local api="https://api.github.com/repos/${REPO}/releases/latest"
  local json
  json="$(curl -fsSL "$api" 2>/dev/null || true)"
  [ -z "$json" ] && return 1
  printf '%s' "$json" \
    | grep -o '"browser_download_url": *"[^"]*"' \
    | sed 's/.*"browser_download_url": *"\([^"]*\)".*/\1/' \
    | grep -iE "$padrao" \
    | head -1
}

instalar_linux() {
  local arq="$1" url tmp
  # AppImage roda em qualquer distro e nao precisa de root.
  url="$(achar_ativo "appimage" || true)"
  if [ -z "$url" ]; then
    return 1
  fi
  tmp="$(mktemp -d)"
  baixar "$url" "$tmp/postly.AppImage"
  chmod +x "$tmp/postly.AppImage"
  local destino="$HOME/.local/bin"
  mkdir -p "$destino"
  mv "$tmp/postly.AppImage" "$destino/postly"
  rm -rf "$tmp"
  msg "${VERDE}Instalado em $destino/postly${ZERA}"
  case ":$PATH:" in
    *":$destino:"*) msg "Rode com: ${FORTE}postly${ZERA}" ;;
    *) msg "Rode com: ${FORTE}$destino/postly${ZERA}"
       msg "${CINZA}(adicione $destino ao PATH para chamar so de 'postly')${ZERA}" ;;
  esac
}

instalar_macos() {
  local arq="$1" url tmp
  url="$(achar_ativo "${arq}.*\.dmg|\.dmg" || true)"
  [ -z "$url" ] && return 1
  tmp="$(mktemp -d)"
  baixar "$url" "$tmp/postly.dmg"
  msg "${CINZA}montando...${ZERA}"
  local ponto
  ponto="$(hdiutil attach -nobrowse -quiet "$tmp/postly.dmg" | tail -1 | awk '{print $NF}')"
  cp -R "$ponto"/*.app /Applications/
  hdiutil detach -quiet "$ponto"
  rm -rf "$tmp"
  msg "${VERDE}Instalado em /Applications${ZERA}"
  msg "${CINZA}Na primeira abertura o macOS pede confirmacao: clique com o botao direito no app e escolha Abrir.${ZERA}"
}

# Windows a partir de um shell POSIX — Git Bash, MSYS2 ou Cygwin.
#
# A release traz dois instaladores para Windows: o NSIS (`-setup.exe`) e o MSI.
# O nome do arquivo baixado tem de acompanhar o que ele é: `msiexec` recusa um
# pacote NSIS, e a extensão errada faz o Windows abrir o programa errado.
instalar_windows() {
  local url tmp arquivo ext caminho
  # O MSI vem primeiro na preferência porque `msiexec` instala em silêncio e
  # devolve código de saída; o NSIS é o plano B.
  url="$(achar_ativo "\.msi$" || true)"
  [ -z "$url" ] || ext="msi"
  if [ -z "$url" ]; then
    url="$(achar_ativo "setup\.exe$|\.exe$" || true)"
    [ -z "$url" ] || ext="exe"
  fi
  [ -z "$url" ] && return 1

  tmp="$(mktemp -d)"
  arquivo="$tmp/postly-setup.$ext"
  baixar "$url" "$arquivo"

  # O Windows não entende `/tmp/...`; o caminho precisa ir em formato nativo.
  caminho="$(cygpath -w "$arquivo" 2>/dev/null || echo "$arquivo")"
  msg "${CINZA}abrindo o instalador...${ZERA}"

  if [ "$ext" = "msi" ]; then
    # `//i` e não `/i`: o MSYS traduz caminhos que começam com uma barra, e o
    # argumento chegaria ao msiexec como `C:/Program Files/i`.
    msiexec //i "$caminho" //qb || \
      erro "Nao consegui abrir o instalador. Ele esta em $caminho"
  else
    # `start` devolve o terminal imediatamente; sem ele o shell fica preso até
    # a pessoa terminar de clicar no instalador.
    cmd //c start "" "$caminho" || \
      erro "Nao consegui abrir o instalador. Ele esta em $caminho"
  fi
}

compilar() {
  msg ""
  msg "${FORTE}Compilando do codigo-fonte.${ZERA}"
  msg "${CINZA}Leva alguns minutos na primeira vez.${ZERA}"
  msg ""

  local faltando=()
  tem git  || faltando+=("git")
  tem node || faltando+=("node (18+)")
  tem cargo|| faltando+=("rust — instale em https://rustup.rs")
  if [ ${#faltando[@]} -gt 0 ]; then
    erro "Faltam: ${faltando[*]}"
    exit 1
  fi

  local destino="${POSTLY_DIR:-$HOME/postly}"
  if [ -d "$destino/.git" ]; then
    msg "Atualizando $destino"
    git -C "$destino" pull --ff-only
  elif [ -e "$destino" ]; then
    # Pasta ocupada por outra coisa. Clonar por cima daria um erro cru do git;
    # melhor dizer o que houve e como escolher outro lugar.
    erro "Ja existe algo em $destino que nao e um clone do Postly."
    erro "Escolha outro lugar:  POSTLY_DIR=~/dev/postly bash instalar.sh"
    exit 1
  else
    git clone --depth 1 "https://github.com/${REPO}.git" "$destino"
  fi

  cd "$destino"
  npm install
  npm install --prefix sidecar
  npm run tauri build

  msg ""
  msg "${VERDE}Pronto.${ZERA} Os pacotes estao em:"
  msg "  $destino/src-tauri/target/release/bundle/"
}

# ── fluxo ───────────────────────────────────────────────────────────────────
main() {
  msg ""
  msg "${VERDE}${FORTE}  Postly${ZERA}"
  msg "${CINZA}  um departamento de marketing que roda na sua maquina${ZERA}"
  msg ""

  local so arq
  so="$(detectar_so)"
  arq="$(detectar_arq)"

  local nomes=("Linux" "macOS" "Windows" "Compilar do codigo-fonte")
  local chaves=("linux" "macos" "windows" "fonte")
  case "$so" in
    linux)   PRE_SELECIONADO=0 ;;
    macos)   PRE_SELECIONADO=1 ;;
    windows) PRE_SELECIONADO=2 ;;
    *)       PRE_SELECIONADO=3 ;;
  esac
  export PRE_SELECIONADO

  menu "Qual e o seu sistema?" "${nomes[@]}"
  local alvo="${chaves[$ESCOLHA]}"
  msg ""

  local ok=0
  case "$alvo" in
    linux)   instalar_linux "$arq" && ok=1 || true ;;
    macos)   instalar_macos "$arq" && ok=1 || true ;;
    windows) instalar_windows && ok=1 || true ;;
    fonte)   compilar; ok=1 ;;
  esac

  if [ "$ok" -eq 0 ]; then
    msg "${CINZA}Nao encontrei pacote pronto para $alvo/$arq na release mais recente.${ZERA}"
    compilar
  fi

  msg ""
  msg "Na primeira abertura o Postly confere a sua maquina e instala o Ollama"
  msg "se ele nao estiver la. Nada mais precisa ser configurado a mao."
  msg ""
}

main "$@"
