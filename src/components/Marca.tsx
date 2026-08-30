/** A marca do Postly.
 *
 *  A haste do P e a rota; o ponto e a posta onde o despacho esta agora. E a
 *  mesma gramatica da trilha de revezamento, reduzida ao tamanho de um icone.
 *
 *  Dentro do trilho o quadrado de fundo sai: verde sobre verde nao se ve, e a
 *  marca fica melhor desenhada direto sobre a superficie. */
export default function Marca({ size = 22, placa = false }: { size?: number; placa?: boolean }) {
  // Sem a placa, o quadrado de 32 deixa uma folga larga a direita do P e a
  // palavra ao lado parece descolada. O viewBox recortado enquadra so o
  // desenho, e a largura acompanha.
  const caixa = placa ? "0 0 32 32" : "9 5.6 14 21.4";
  const largura = placa ? size : Math.round(size * (14 / 21.4));
  return (
    <svg width={largura} height={size} viewBox={caixa} aria-hidden focusable="false">
      {placa && <rect width="32" height="32" rx="7.5" fill="var(--act)" />}
      <path
        d="M11.6 24.5V9.4h5.1a3.9 3.9 0 0 1 0 7.8h-5.1"
        fill="none"
        stroke={placa ? "var(--bg)" : "currentColor"}
        strokeWidth="3.3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="11.6" cy="9.4" r="2.9" fill="var(--act)" />
    </svg>
  );
}
