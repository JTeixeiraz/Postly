#!/usr/bin/env bash
# Apaga todo o estado do Postly, para a proxima abertura ser a primeira.
#
# Some: preferencias, identidade da marca, referencias, cofre com a chave do
# Gemini e as credenciais, cerebro em grafo, historico de campanhas, perfis do
# navegador e o localStorage da janela (que guarda o idioma escolhido).
#
# NAO some: o Ollama nem os modelos baixados. Eles sao do sistema, nao do app,
# e rebaixar 20 GB para testar uma tela nao seria um favor. Para testar o fluxo
# de instalacao do Ollama, veja a nota no fim.

set -euo pipefail

APP_ID="dev.teixas.postly"

case "$(uname -s)" in
  Darwin)
    DADOS="$HOME/Library/Application Support/postly"
    WEBVIEW="$HOME/Library/WebKit/$APP_ID"
    ;;
  Linux)
    DADOS="${XDG_DATA_HOME:-$HOME/.local/share}/postly"
    WEBVIEW="${XDG_DATA_HOME:-$HOME/.local/share}/$APP_ID"
    ;;
  *)
    # Git Bash / MSYS no Windows.
    DADOS="${APPDATA:-$HOME/AppData/Roaming}/postly"
    WEBVIEW="${LOCALAPPDATA:-$HOME/AppData/Local}/$APP_ID"
    ;;
esac

echo "Estado do Postly:"
for alvo in "$DADOS" "$WEBVIEW"; do
  if [ -e "$alvo" ]; then
    echo "  $alvo  ($(du -sh "$alvo" 2>/dev/null | cut -f1))"
  else
    echo "  $alvo  (nao existe)"
  fi
done

if [ "${1:-}" != "--sim" ]; then
  echo
  echo "Isto apaga chave do Gemini, credenciais, cerebro e historico."
  printf "Apagar? [s/N] "
  read -r resposta
  case "$resposta" in
    s|S|y|Y) ;;
    *) echo "Cancelado."; exit 0 ;;
  esac
fi

rm -rf "$DADOS" "$WEBVIEW"
echo "Pronto. A proxima abertura vai ser a primeira."
echo
echo "Para testar TAMBEM a instalacao automatica do Ollama, remova o binario:"
echo "  Linux (Arch):  sudo pacman -R ollama"
echo "  Linux (outro): sudo rm -f /usr/local/bin/ollama"
echo "  macOS:         brew uninstall ollama"
echo "  Windows:       winget uninstall Ollama.Ollama"
