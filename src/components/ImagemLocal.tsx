import { useCallback, useEffect, useState } from "react";
import { motion } from "motion/react";
import { api, ouvirImagemLocal } from "../api";
import { formatarBytes, useIdioma } from "../i18n";
import type { EstadoLocal, ProgressoLocal } from "../types";
import { IconDownload, IconTrash } from "./Icons";

/** Difusão na própria máquina.
 *
 *  O único provedor de arte que não pede chave e não manda nada para fora —
 *  com ele e o Ollama, a campanha inteira roda sem que uma linha saia do
 *  computador.
 *
 *  Nada vem instalado, e o motivo está à vista: o motor e um modelo somam
 *  gigabytes. Baixar isso sem a pessoa pedir seria decidir por ela o que fazer
 *  com o disco e com a banda dela. */
export default function ImagemLocal() {
  const { d, f, idioma } = useIdioma();
  const [estado, setEstado] = useState<EstadoLocal | null>(null);
  const [baixando, setBaixando] = useState<ProgressoLocal | null>(null);
  const [erro, setErro] = useState<string | null>(null);

  const ler = useCallback(() => {
    api.estadoImagemLocal().then(setEstado).catch(() => {});
  }, []);

  useEffect(ler, [ler]);

  useEffect(() => {
    const p = ouvirImagemLocal(setBaixando);
    return () => void p.then((u) => u());
  }, []);

  const baixar = async (fn: () => Promise<unknown>) => {
    setErro(null);
    try {
      await fn();
    } catch (e) {
      setErro(String(e));
    }
    setBaixando(null);
    ler();
  };

  if (!estado) return null;
  const temMotor = !!estado.motor;

  return (
    <div className="stack">
      {/* O motor vem primeiro porque sem ele nenhum modelo roda. Mostrar os
          dois lado a lado convidaria a baixar 2 GB de pesos que não teriam
          o que os executasse. */}
      <div className="row row--tight">
        <strong style={{ color: "var(--ink)" }}>{d.local.motor}</strong>
        {temMotor ? (
          <span className="tag" data-tone="ok">
            <span className="tag__dot" />
            {d.local.instalado}
          </span>
        ) : (
          <button
            className="btn btn--sm btn--key push"
            disabled={!!baixando}
            onClick={() => void baixar(() => api.baixarMotorLocal())}
          >
            <IconDownload size={14} />
            {d.local.baixarMotor}
          </button>
        )}
      </div>
      <p className="hint">{d.local.motorPorque}</p>

      {temMotor && (
        <>
          <div className="modelos-local">
            {estado.modelos.map((m) => (
              <div className="modelo-local" key={m.id} data-on={m.baixado}>
                <div className="row row--tight">
                  <strong style={{ color: "var(--ink)" }}>{m.nome}</strong>
                  <span className="hint mono">
                    {formatarBytes(m.bytes, idioma)} ·{" "}
                    {f(d.local.passos, { n: m.passos, px: m.base })}
                  </span>
                  <span className="push" />
                  {m.baixado ? (
                    <button
                      className="btn btn--quiet btn--sm"
                      title={d.local.remover}
                      onClick={() => void baixar(() => api.removerModeloLocal(m.id))}
                    >
                      <IconTrash size={14} />
                    </button>
                  ) : (
                    <button
                      className="btn btn--sm"
                      disabled={!!baixando}
                      onClick={() => void baixar(() => api.baixarModeloLocal(m.id))}
                    >
                      <IconDownload size={14} />
                      {d.local.baixar}
                    </button>
                  )}
                </div>
                <p className="hint">{m.nota}</p>

                {baixando?.alvo === m.id && (
                  <div className="provisao__barra">
                    <motion.span
                      className="provisao__preenche"
                      animate={{ scaleX: baixando.percent / 100 }}
                      transition={{ duration: 0.2 }}
                    />
                  </div>
                )}
              </div>
            ))}
          </div>

          {estado.bytes_em_disco > 0 && (
            <p className="hint">
              {f(d.local.emDisco, { n: formatarBytes(estado.bytes_em_disco, idioma) })}
            </p>
          )}
        </>
      )}

      {baixando && (
        <p className="hint mono">
          {f(d.local.baixandoAgora, {
            alvo: baixando.alvo === "motor" ? d.local.motor : baixando.alvo,
            p: baixando.percent,
            n: formatarBytes(baixando.baixado, idioma),
            t: formatarBytes(baixando.total, idioma),
          })}
        </p>
      )}

      {erro && (
        <p className="hint" data-alerta="true">
          {erro}
        </p>
      )}
    </div>
  );
}
