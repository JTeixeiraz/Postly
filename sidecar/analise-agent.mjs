// Análise dos clipes que a pessoa subiu, chamada pelo Rust.
//
// O QUE ISTO RESPONDE, e por que o modelo não consegue responder sozinho: onde
// há silêncio num vídeo. Um modelo de linguagem recebe texto; ele não ouve o
// arquivo. Sem esta medição, "corte as pausas vazias" seria um pedido que ele
// só poderia fingir atender — e fingiria, cortando em tempos inventados.
//
// Aqui a medição é real e vem do próprio Remotion: `getSilentParts` roda no
// compositor que já está instalado para renderizar. Nenhum ffmpeg externo,
// nenhum download novo.
//
// Um processo por análise, como o render: é operação longa e única.

import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";

const emitir = (obj) => process.stdout.write(`${JSON.stringify(obj)}\n`);

/** Abaixo de quantos decibéis conta como silêncio.
 *
 *  -30 dB e não 0: um take real tem ruído de sala, respiração e o zumbido do
 *  ar-condicionado. Exigir silêncio absoluto não acharia pausa nenhuma no
 *  material que as pessoas de fato gravam. */
const RUIDO_DB = -30;

/** A pausa mais curta que vale cortar, em segundos.
 *
 *  Abaixo de 0,4s não é pausa, é respiração entre palavras — e cortar ali
 *  produz fala picotada, que é pior que a pausa. */
const PAUSA_MINIMA_S = 0.4;

async function main() {
  const { projeto } = JSON.parse(await lerEntrada());

  let getVideoMetadata, getSilentParts;
  try {
    ({ getVideoMetadata, getSilentParts } = await import("@remotion/renderer"));
  } catch (e) {
    return emitir({
      ok: false,
      erro:
        "O analisador não está instalado no sidecar. Rode `npm ci --prefix sidecar`. " +
        String(e?.message ?? e),
    });
  }

  const dir = join(projeto, "clipes");
  if (!existsSync(dir)) return emitir({ ok: true, clipes: [] });

  const nomes = readdirSync(dir).sort();
  const clipes = [];

  for (const [i, nome] of nomes.entries()) {
    emitir({ fase: "analisando", percent: i / Math.max(nomes.length, 1), detalhe: nome });
    const src = join(dir, nome);
    try {
      const meta = await getVideoMetadata(src);
      const som = await getSilentParts({
        src,
        noiseThresholdInDecibels: RUIDO_DB,
        minDurationInSeconds: PAUSA_MINIMA_S,
      });
      clipes.push({
        nome,
        duracao_s: meta.durationInSeconds ?? som.durationInSeconds ?? 0,
        largura: meta.width,
        altura: meta.height,
        fps: meta.fps,
        tem_audio: meta.audioCodec !== null,
        // Os trechos COM som, que é o que o modelo precisa para montar. Mandar
        // os silêncios obrigaria ele a subtrair intervalos de cabeça, e é
        // exatamente esse tipo de conta que um modelo pequeno erra.
        com_som: som.audibleParts.map((p) => ({
          de_s: Number(p.startInSeconds.toFixed(2)),
          ate_s: Number(p.endInSeconds.toFixed(2)),
        })),
        pausas: som.silentParts.length,
      });
    } catch (e) {
      // Um clipe ilegível não derruba os outros: quem subiu cinco vídeos com um
      // corrompido no meio quer os quatro que servem, com aviso sobre o quinto.
      clipes.push({ nome, erro: String(e?.message ?? e) });
    }
  }

  emitir({ ok: true, clipes });
}

function lerEntrada() {
  return new Promise((resolve, reject) => {
    let buf = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (c) => (buf += c));
    process.stdin.on("end", () => resolve(buf.trim()));
    process.stdin.on("error", reject);
  });
}

main().catch((e) => {
  emitir({ ok: false, erro: String(e?.message ?? e) });
  process.exitCode = 1;
});
