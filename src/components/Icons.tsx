// Conjunto proprio de icones.
//
// Traco 1.5 em todos, cor herdada, 24x24. Um pacote de icones traria centenas
// que este app nunca usa; aqui sao doze, e eles combinam entre si por
// construcao e nao por sorte.

type Props = { size?: number; className?: string };

const base = (size: number, className?: string) => ({
  width: size,
  height: size,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.5,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  className,
  "aria-hidden": true,
});

export const IconChip = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <rect x="7" y="7" width="10" height="10" rx="1.5" />
    <path d="M4 10h3M4 14h3M17 10h3M17 14h3M10 4v3M14 4v3M10 17v3M14 17v3" />
  </svg>
);

export const IconLayers = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M12 3 3 7.5 12 12l9-4.5L12 3Z" />
    <path d="m3 12.5 9 4.5 9-4.5M3 17l9 4.5 9-4.5" />
  </svg>
);

export const IconRelay = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <circle cx="5" cy="12" r="2" />
    <circle cx="19" cy="12" r="2" />
    <path d="M7.5 12h5.5m0 0-2-2m2 2-2 2" />
  </svg>
);

/** Uma claquete: dois retângulos e a barra diagonal.
 *
 *  Filme, e não uma seta de "play". O play significaria "assistir", e esta aba
 *  é onde o vídeo é FEITO — a distinção importa numa barra onde as outras cinco
 *  abas também são lugares de trabalho. */
export const IconFilm = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <rect x="3" y="8" width="18" height="12" rx="2" />
    <path d="M3.6 8 6.8 4h3.4L7 8m4.2 0L14.4 4h3.4L14.6 8" />
  </svg>
);

export const IconGraph = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <circle cx="6" cy="7" r="2.2" />
    <circle cx="18" cy="6" r="2.2" />
    <circle cx="12" cy="17" r="2.2" />
    <path d="m7.9 8.4 2.6 6.7M8.2 7 15.9 6.2M16.8 8l-3.6 7" />
  </svg>
);

export const IconArchive = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <rect x="3" y="4" width="18" height="4" rx="1" />
    <path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8M10 12h4" />
  </svg>
);

export const IconCheck = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="m4 12.5 5 5L20 6.5" />
  </svg>
);

export const IconAlert = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M12 4.5 2.8 20h18.4L12 4.5Z" />
    <path d="M12 10v4.5M12 17.4v.1" />
  </svg>
);

export const IconSpinner = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)} className={`girar ${className ?? ""}`}>
    <path d="M12 3a9 9 0 1 0 9 9" />
  </svg>
);

export const IconDot = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <circle cx="12" cy="12" r="3.2" />
  </svg>
);

export const IconDownload = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M12 3.5v11m0 0-4-4m4 4 4-4M4 18.5h16" />
  </svg>
);

export const IconOpen = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M14 4h6v6M20 4l-8.5 8.5" />
    <path d="M18 14.5V19a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h4.5" />
  </svg>
);

export const IconArrow = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M4.5 12h15m0 0-5.5-5.5M19.5 12 14 17.5" />
  </svg>
);

export const IconBroom = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M15.5 3.5 20 8M17.8 5.8 9.5 14M4 20l3-6 7 7-6 3-4-4Z" />
  </svg>
);

export const IconTrash = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M4 6h16M9 6V4.5A1.5 1.5 0 0 1 10.5 3h3A1.5 1.5 0 0 1 15 4.5V6" />
    <path d="M6.5 6l.8 12.2A1.8 1.8 0 0 0 9.1 20h5.8a1.8 1.8 0 0 0 1.8-1.8L17.5 6" />
  </svg>
);

export const IconSliders = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M4 7h10M18 7h2M4 17h4M12 17h8" />
    <circle cx="16" cy="7" r="2" />
    <circle cx="10" cy="17" r="2" />
  </svg>
);

export const IconHelp = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <circle cx="12" cy="12" r="9" />
    <path d="M9.6 9.4a2.5 2.5 0 1 1 3.3 2.4c-.6.2-.9.7-.9 1.3v.6" />
    <path d="M12 16.8h.01" />
  </svg>
);

/** Ponteiro de mostrador. A auditoria e a unica tela que mede resultado, e
 *  medidor e o simbolo que diz isso sem precisar de rotulo. */
export const IconGauge = ({ size = 18, className }: Props) => (
  <svg {...base(size, className)}>
    <path d="M3.2 15a8 8 0 1 1 15.6 0" />
    <path d="M11 15l4.2-5" />
    <circle cx="11" cy="15" r="1.4" />
  </svg>
);
