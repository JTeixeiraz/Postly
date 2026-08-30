//! Testes da deteccao de hardware e da escolha de modelo por cargo.
//!
//! Rodam contra a maquina de verdade: o valor deles e garantir que as regras
//! de proporcionalidade e de orcamento valem para o hardware que existe, e nao
//! so para numeros inventados.

use postly_lib::hardware::{self, accelerator::ComputeMode};
use postly_lib::ollama::catalog;
use postly_lib::orchestrator::roles::{Role, Tier};
use postly_lib::platform;

#[test]
fn a_estrategia_do_so_resolve_para_a_plataforma_corrente() {
    let s = platform::current();
    #[cfg(target_os = "linux")]
    assert_eq!(s.id(), platform::Platform::Linux);
    #[cfg(target_os = "windows")]
    assert_eq!(s.id(), platform::Platform::Windows);
    #[cfg(target_os = "macos")]
    assert_eq!(s.id(), platform::Platform::MacOS);

    assert!(!s.ollama_install_steps().is_empty(), "todo SO precisa de um plano de instalacao");
    assert!(s.data_dir().is_absolute(), "o diretorio de dados precisa ser absoluto");
}

#[test]
fn a_leitura_de_memoria_e_coerente() {
    let ram = hardware::snapshot();
    assert!(ram.total_bytes > 0, "nao consegui ler a RAM total");
    assert!(ram.available_bytes <= ram.total_bytes);
    assert!(ram.max_budget_bytes < ram.total_bytes, "o teto precisa deixar folga para o SO");
    assert!(
        ram.live_budget_bytes <= ram.available_bytes,
        "o orcamento de runtime nao pode passar da memoria livre"
    );
    println!(
        "maquina: {} totais, {} livres, teto de modelo {}",
        hardware::human(ram.total_bytes),
        hardware::human(ram.available_bytes),
        hardware::human(ram.max_budget_bytes)
    );
}

#[test]
fn o_catalogo_separa_o_que_a_maquina_aguenta() {
    let perfil = hardware::compute_profile();
    let entradas = catalog::build(&perfil, &[]);
    assert!(!entradas.is_empty());

    for e in &entradas {
        assert_eq!(
            e.supported,
            e.footprint_bytes <= perfil.max_budget_bytes,
            "{} foi classificado errado", e.spec.tag
        );
        assert!(!e.reason.is_empty(), "toda entrada precisa explicar o proprio veredito");
    }

    // Suportados vem primeiro na lista que a tela renderiza.
    let primeiro_fora = entradas.iter().position(|e| !e.supported);
    if let Some(i) = primeiro_fora {
        assert!(
            entradas[i..].iter().all(|e| !e.supported),
            "a ordenacao precisa agrupar os suportados no topo"
        );
    }

    // O Kimi e da familia priorizada, mas nao roda em hardware de mesa.
    let kimi = entradas.iter().find(|e| e.spec.tag.starts_with("kimi")).expect("kimi no catalogo");
    assert!(kimi.spec.focus, "kimi foi pedido explicitamente e precisa aparecer");
    assert!(!kimi.supported, "1T de parametros nao cabe em hardware de mesa");
}

#[test]
fn o_nivel_do_modelo_e_proporcional_ao_cargo() {
    // Orcamento generoso: cada cargo deve pegar o topo do SEU nivel, nao o topo geral.
    let farto = 40u64 * 1024 * 1024 * 1024;

    let (gerente, _) = catalog::pick(Role::GerenteSetor.tier(), farto, ComputeMode::Dedicada, false, &[]).unwrap();
    let (auditor, _) = catalog::pick(Role::Auditor.tier(), farto, ComputeMode::Dedicada, false, &[]).unwrap();
    let (criador, _) = catalog::pick(Role::Criador.tier(), farto, ComputeMode::Dedicada, false, &[]).unwrap();

    assert!(
        gerente.strength > auditor.strength,
        "quem decide ({}) precisa ser mais forte que quem audita ({})",
        gerente.label, auditor.label
    );
    assert!(
        auditor.strength > criador.strength,
        "quem audita ({}) precisa ser mais forte que quem executa ({})",
        auditor.label, criador.label
    );
    assert_eq!(Role::Criador.tier(), Tier::Baixo);
    assert_eq!(Role::DiretorGeral.tier(), Tier::Alto);
}

#[test]
fn o_kimi_nunca_e_escolhido_para_rodar() {
    // Mesmo com orcamento absurdo, um modelo de 600 GB nao pode ser sorteado
    // em nenhuma maquina real.
    let real = 64u64 * 1024 * 1024 * 1024;
    let (escolhido, _) = catalog::pick(Tier::Alto, real, ComputeMode::Dedicada, false, &[]).unwrap();
    assert!(!escolhido.tag.starts_with("kimi"), "escolheu {}", escolhido.tag);
}

#[test]
fn falta_de_memoria_rebaixa_o_cargo_com_aviso() {
    // 5 GB nao comportam nenhum modelo de nivel alto.
    let apertado = 5u64 * 1024 * 1024 * 1024;
    let (modelo, aviso) = catalog::pick(Tier::Alto, apertado, ComputeMode::Cpu, false, &[]).expect("precisa rebaixar, nao falhar");

    assert!(modelo.footprint_bytes() <= apertado, "escolheu algo que nao cabe");
    assert!(aviso.is_some(), "o rebaixamento precisa ser registrado para o usuario ver");
    assert_ne!(modelo.tier, Tier::Alto);
}

#[test]
fn sem_memoria_nenhuma_a_escolha_falha_em_vez_de_estourar() {
    let nada = 200u64 * 1024 * 1024;
    assert!(catalog::pick(Tier::Alto, nada, ComputeMode::Cpu, false, &[]).is_none());
}

#[test]
fn o_organograma_impede_atalho_entre_cargos() {
    assert!(Role::DiretorGeral.can_send_to(Role::GerenteSetor));
    assert!(!Role::DiretorGeral.can_send_to(Role::Criador), "o diretor nao fala direto com o executor");
    assert!(Role::GerenteSetor.can_send_to(Role::Criador));
    assert!(Role::Criador.can_send_to(Role::Auditor));
    assert!(!Role::Criador.can_send_to(Role::GerenteSetor), "o criador nao responde ao gerente por conta propria");
    assert!(Role::Auditor.can_send_to(Role::GerenteSetor), "a decisao final e conjunta com quem manda");
}

#[test]
fn a_escolha_muda_com_o_hardware() {
    // Mesmo orcamento, mesma exigencia de cargo, maquinas diferentes.
    let orcamento = 26u64 * 1024 * 1024 * 1024;

    let (com_gpu, _) = catalog::pick(Tier::Alto, orcamento, ComputeMode::Dedicada, false, &[]).unwrap();
    let (so_cpu, _) = catalog::pick(Tier::Alto, orcamento, ComputeMode::Cpu, false, &[]).unwrap();

    // Com GPU, capacidade bruta manda: o denso mais forte vence.
    assert!(!com_gpu.moe, "com GPU esperava o denso mais capaz, veio {}", com_gpu.label);

    // Sem GPU, o que decide e o custo por token: o MoE precisa ganhar.
    assert!(
        so_cpu.moe,
        "sem GPU o MoE tinha de vencer o denso, veio {} ({}B ativos)",
        so_cpu.label, so_cpu.active_params_b
    );
    assert!(
        so_cpu.active_params_b < com_gpu.active_params_b,
        "a escolha de CPU precisa ter menos parametros ativos"
    );
    println!("GPU -> {} | CPU -> {}", com_gpu.label, so_cpu.label);
}

#[test]
fn o_perfil_desta_maquina_e_coerente() {
    let p = hardware::compute_profile();
    assert!(p.max_budget_bytes >= p.ram.max_budget_bytes);
    assert!(p.throughput_constant > 0.0);

    // Memoria unificada nunca soma VRAM ao total: seria contar duas vezes.
    if p.mode == ComputeMode::Unificada {
        assert_eq!(p.vram_total_bytes, 0);
    }
    if p.mode == ComputeMode::Cpu {
        assert_eq!(p.accelerated_budget_bytes, 0);
        assert!(p.prefers_moe, "sem acelerador o sistema tem de preferir MoE");
    }

    println!(
        "modo: {} | acelerador: {} | placas vistas: {}",
        p.mode_label,
        p.primary_name.clone().unwrap_or_else(|| "nenhum".into()),
        p.accelerators
            .iter()
            .map(|a| format!("{} ({}, {})", a.name, hardware::human(a.vram_total_bytes),
                if a.usable { "utilizavel" } else { "sem driver" }))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// O percentual da barra vem de texto de instalador, e cada um escreve de um
/// jeito. Estas sao linhas reais dos tres caminhos que o app usa.
#[test]
fn a_barra_le_o_percentual_dos_tres_instaladores() {
    use postly_lib::ollama::installer::percentual_publico as pct;

    // curl, usado pelo script oficial do Ollama (Linux e macOS sem brew).
    assert_eq!(pct("######                     8,7%"), Some(8.7));
    assert_eq!(pct("################# 45.2%"), Some(45.2));
    assert_eq!(pct("100%"), Some(100.0));

    // winget, no Windows.
    assert_eq!(pct("  ██████████████  73%"), Some(73.0));

    // pacman e brew nao emitem percentual: a barra corre sem numero em vez de
    // fingir um progresso que nao existe.
    assert_eq!(pct("resolving dependencies..."), None);
    assert_eq!(pct("==> Downloading ollama"), None);

    // Lixo nao vira numero.
    assert_eq!(pct("%"), None);
    assert_eq!(pct("abc% def"), None);
    assert_eq!(pct("999%"), None);
}


// ---------------------------------------------------------------- movimento

/// O parser da declaracao de movimento decide se um turno inteiro roda. Errar
/// aqui nao da erro nenhum: ou o recurso some em silencio, ou a campanha para
/// para perguntar sobre uma peca que ninguem quis animar.
#[test]
fn a_declaracao_de_movimento_e_lida_como_o_gerente_escreve() {
    use postly_lib::orchestrator::movimento::motivo_do_movimento;

    // O caso feliz, exatamente no formato pedido.
    let m = motivo_do_movimento("briefing...\nMOVIMENTO: sim - o numero precisa subir na tela");
    assert_eq!(m.as_deref(), Some("o numero precisa subir na tela"));

    // Modelo pequeno escreve com caixa e travessao proprios. Exigir o formato
    // exato faria o recurso falhar na metade do catalogo.
    assert!(motivo_do_movimento("Movimento: Sim \u{2014} transformacao antes e depois").is_some());

    // Linha depois da declaracao: a busca vai de tras para frente, mas nao pode
    // parar na primeira linha qualquer.
    assert!(
        motivo_do_movimento("MOVIMENTO: sim - vale\n\nObrigado!").is_some(),
        "uma linha extra depois da declaracao nao pode apagar a decisao"
    );

    // Nao e nao, em qualquer grafia.
    for nao in ["MOVIMENTO: nao", "movimento: não", "MOVIMENTO: no"] {
        assert!(motivo_do_movimento(nao).is_none(), "deveria recusar: {nao}");
    }

    // Sem a linha, nao ha pedido: um briefing antigo nao pode acionar o turno.
    assert!(motivo_do_movimento("briefing sem a linha final").is_none());

    // Sim sem motivo ainda vale: quem decidiu foi o gerente, e a pessoa
    // confirma depois de qualquer jeito.
    assert!(motivo_do_movimento("MOVIMENTO: sim").is_some());
}
