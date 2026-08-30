/** O glifo do Claude.
 *
 *  Doze raios saindo de um centro, com comprimentos desiguais — é o que dá ao
 *  símbolo a aparência de brilho em vez de asterisco. Desenhado em SVG e não
 *  trazido como arquivo de imagem: assim ele herda a cor do contexto e fica
 *  nítido em qualquer densidade de tela.
 *
 *  Uso nominativo: indica que o Postly conversa com o Claude Code. A marca é
 *  da Anthropic, e a ressalva ao pé da seção diz isso em palavras. */
export default function MarcaClaude({ size = 20 }: { size?: number }) {
  // Comprimento de cada raio, a partir do centro, em ordem angular. A
  // irregularidade é do símbolo, não ruído: raios iguais viram asterisco.
  const raios = [8.4, 5.2, 7.6, 4.6, 8.4, 5.8, 7.9, 4.9, 8.1, 5.4, 7.6, 4.8];
  const r0 = 1.6;

  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden focusable="false">
      {raios.map((comp, i) => {
        const a = (i * Math.PI * 2) / raios.length - Math.PI / 2;
        const [cx, cy] = [12, 12];
        return (
          <line
            key={i}
            x1={cx + Math.cos(a) * r0}
            y1={cy + Math.sin(a) * r0}
            x2={cx + Math.cos(a) * (r0 + comp)}
            y2={cy + Math.sin(a) * (r0 + comp)}
            stroke="currentColor"
            strokeWidth={i % 2 === 0 ? 2.2 : 1.7}
            strokeLinecap="round"
          />
        );
      })}
    </svg>
  );
}
