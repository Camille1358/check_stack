use colored::*;
use reqwest::blocking::Client;
use std::process::Command;
use std::time::{Duration, Instant};

struct ServiceTest {
    name: &'static str,
    url: &'static str,
    port: u16,
    keyword: &'static str,
    auth_header: Option<(&'static str, &'static str)>, // <-- AJOUT : En-tête d'authentification optionnel
    systemd_service: Option<&'static str>,            // <-- AJOUT : Nom du service systemd si non-Docker
}

struct TestResult {
    name: &'static str,
    port: u16,
    keyword: &'static str,
    status: &'static str,
    latency_ms: u128,
    details: String,
}

struct DockerContainerInfo {
    id: String,
    name: String,
    status: String,
    ports: String,
}

fn main() {
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{}", "                      INSPECTION GLOBALE DE LA STACK IA LOCAL                     ".bold().cyan());
    println!("{}", "==========================================================================================".bold().cyan());
    println!();

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Échec de création du client HTTP");

    let services = vec![
        ServiceTest { name: "Ollama (N1.1)", url: "http://127.0.0.1:11434/api/version", port: 11434, keyword: "ollama", auth_header: None, systemd_service: None },
        ServiceTest { name: "Mistral.rs (N1.2)", url: "http://127.0.0.1:1234/v1/models", port: 1234, keyword: "mistral", auth_header: None, systemd_service: None },
        ServiceTest { name: "Llama.cpp (N1.3)", url: "http://127.0.0.1:8085/v1/models", port: 8085, keyword: "llama", auth_header: None, systemd_service: None },
        ServiceTest { name: "Qdrant Vector DB (N2.1)", url: "http://127.0.0.1:6333/healthz", port: 6333, keyword: "qdrant", auth_header: None, systemd_service: None },
        ServiceTest { name: "TEI Embeddings (N2.3)", url: "http://127.0.0.1:8080/health", port: 8080, keyword: "tei", auth_header: None, systemd_service: None },
        
        // Mem0 : Service systemd natif
        ServiceTest { name: "Mem0 Memory (N2.4)", url: "http://127.0.0.1:8081/health", port: 8081, keyword: "mem0", auth_header: None, systemd_service: Some("mem0-service.service") },
        
        ServiceTest { name: "LiteLLM Proxy (N3.2)", url: "http://127.0.0.1:4000/health/readiness", port: 4000, keyword: "litellm", auth_header: None, systemd_service: None },
        
        // Guardrails AI : Service systemd natif
        ServiceTest { name: "Guardrails AI (N4.2)", url: "http://127.0.0.1:8005/health", port: 8005, keyword: "guardrails", auth_header: None, systemd_service: Some("guardrails-api.service") },
        
        // OmniRoute : Nécessite l'en-tête Authorization
        ServiceTest { name: "OmniRoute Façade (N3.1)", url: "http://127.0.0.1:3000/api/health", port: 3000, keyword: "omniroute", auth_header: None, systemd_service: None },
        
        ServiceTest { name: "Langfuse Tracing (N5.4)", url: "http://127.0.0.1:3001/api/public/health", port: 3001, keyword: "langfuse", auth_header: None, systemd_service: None },
        ServiceTest { name: "SearXNG Search (N6.3)", url: "http://127.0.0.1:8888/search?q=test&format=json", port: 8888, keyword: "searxng", auth_header: None, systemd_service: None },
        ServiceTest { name: "Open WebUI (N7.1)", url: "http://127.0.0.1:8082", port: 8082, keyword: "open-webui", auth_header: None, systemd_service: None },
        ServiceTest { name: "n8n Workflows (N7.4)", url: "http://127.0.0.1:5678", port: 5678, keyword: "n8n", auth_header: None, systemd_service: None },
    ];

    let mut results = Vec::new();

    println!("{}", "[LOGS DETAILLES DE NAVIGATION ET REPONSES]".bold().yellow());
    println!("{}", "------------------------------------------------------------------------------------------".dimmed());

    for service in &services {
        print!("[TEST] Interrogation de {:<22} ({:<38}) ... ", service.name.bold(), service.url);
        let start = Instant::now();

        // MODIFICATION : Prise en compte de l'en-tête d'authentification s'il est présent
        let mut req = client.get(service.url);
        if let Some((header_key, header_val)) = service.auth_header {
            req = req.header(header_key, header_val);
        }

        let response = req.send();
        let duration = start.elapsed().as_millis();

        match response {
            Ok(resp) => {
                let code = resp.status();
                if code.is_success() || code.is_redirection() {
                    println!("{}", format!("SUCCES ({}) - {}ms", code, duration).green());
                    results.push(TestResult {
                        name: service.name,
                        port: service.port,
                        keyword: service.keyword,
                        status: "ACTIF",
                        latency_ms: duration,
                        details: format!("HTTP {}", code),
                    });
                } else {
                    println!("{}", format!("REPONSE ANORMALE ({}) - {}ms", code, duration).yellow());
                    results.push(TestResult {
                        name: service.name,
                        port: service.port,
                        keyword: service.keyword,
                        status: "PARTIEL",
                        latency_ms: duration,
                        details: format!("HTTP {}", code),
                    });
                }
            }
            Err(err) => {
                let err_msg = if err.is_timeout() {
                    "Timeout (3s)"
                } else if err.is_connect() {
                    "Port fermé / Injoignable"
                } else {
                    "Erreur Réseau"
                };

                println!("{}", format!("ECHEC ({})", err_msg).red());
                results.push(TestResult {
                    name: service.name,
                    port: service.port,
                    keyword: service.keyword,
                    status: "INACTIF",
                    latency_ms: 0,
                    details: err_msg.to_string(),
                });
            }
        }
    }

    println!("\n");
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{}", "                                 TABLEAU RECAPITULATIF                                   ".bold().cyan());
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{:<26} | {:<7} | {:<10} | {:<10} | {:<25}", "SERVICE", "PORT", "STATUT", "LATENCE", "DETAILS");
    println!("{}", "------------------------------------------------------------------------------------------".cyan());

    for res in &results {
        let status_formatted = match res.status {
            "ACTIF" => "ACTIF   ".green().bold(),
            "PARTIEL" => "PARTIEL ".yellow().bold(),
            _ => "INACTIF ".red().bold(),
        };

        let latence_str = if res.status == "INACTIF" {
            "-".to_string()
        } else {
            format!("{} ms", res.latency_ms)
        };

        println!(
            "{:<26} | {:<7} | {} | {:<10} | {:<25}",
            res.name,
            res.port,
            status_formatted,
            latence_str,
            res.details
        );
    }
    println!("{}", "==========================================================================================".bold().cyan());

    // --- SECTION DIAGNOSTIC ET DEPANNAGE DOCKER (Services non opérationnels uniquement) ---
    let failed_services: Vec<&TestResult> = results.iter().filter(|r| r.status != "ACTIF").collect();

    if !failed_services.is_empty() {
        println!("\n");
        println!("{}", "==========================================================================================".bold().red());
        println!("{}", "                    DIAGNOSTICS & LOGS DOCKER DES SERVICES EN ECHEC                       ".bold().red());
        println!("{}", "==========================================================================================".bold().red());

        for failed in failed_services {
            println!("\n{}", format!(">>> SERVICE EN ECHEC : {} (Port {})", failed.name, failed.port).bold().red());

            // Trouve le service correspondant dans le vecteur d'origine
            let original_service = services.iter().find(|s| s.name == failed.name);

            if let Some(srv) = original_service {
                if let Some(sys_unit) = srv.systemd_service {
                    println!("    └─ Type : Service Systemd Natif ({})", sys_unit.bold().cyan());
                    println!("{}", "    ┌────────────────── DERNIERS LOGS SYSTEMD (25 lignes) ──────────────────".dimmed());

                    let output = Command::new("journalctl")
                        .args(["-u", sys_unit, "-n", "25", "--no-pager"])
                        .output();

                    if let Ok(out) = output {
                        let logs = String::from_utf8_lossy(&out.stdout);
                        for line in logs.lines() {
                            println!("    │ {}", line);
                        }
                    } else {
                        println!("    │ Erreur lors de la récupération des logs Systemd.");
                    }
                    println!("{}", "    └───────────────────────────────────────────────────────────────────────".dimmed());
                    continue;
                }
            }

            // Fallback diagnostic Docker pour les conteneurs
            match find_docker_container(failed.port, failed.keyword) {
                Some(container) => {
                    println!("    └─ Conteneur trouvé : {} (ID: {})", container.name.bold(), container.id);
                    println!("    └─ État actuel     : {}", container.status.yellow());
                    println!("    └─ Mapping Ports   : {}", container.ports);
                    println!("{}", "    ┌────────────────── DERNIERS LOGS DOCKER (25 lignes) ──────────────────".dimmed());

                    let logs = fetch_container_logs(&container.id, 25);
                    for line in logs.lines() {
                        println!("    │ {}", line);
                    }
                    println!("{}", "    └───────────────────────────────────────────────────────────────────────".dimmed());
                }
                None => {
                    println!(
                        "    {}",
                        format!(
                            "└─ Aucun conteneur Docker correspondant au port {} ou au nom '{}' n'a été trouvé.",
                            failed.port, failed.keyword
                        )
                        .yellow()
                    );
                }
            }
        }
        println!("\n{}", "==========================================================================================".bold().red());
    }
}

/// Recherche de manière adaptative un conteneur Docker associé au port ou au mot-clé du service.
fn find_docker_container(port: u16, keyword: &str) -> Option<DockerContainerInfo> {
    let output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.ID}}|{{.Names}}|{{.Status}}|{{.Ports}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let port_pattern_1 = format!(":{port}->");
    let port_pattern_2 = format!(":{port}/");

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 4 {
            let id = parts[0].trim();
            let c_name = parts[1].trim();
            let status = parts[2].trim();
            let ports = parts[3].trim();

            let port_match = ports.contains(&port_pattern_1) || ports.contains(&port_pattern_2);
            let name_match = c_name.to_lowercase().contains(&keyword.to_lowercase());

            if port_match || name_match {
                return Some(DockerContainerInfo {
                    id: id.to_string(),
                    name: c_name.to_string(),
                    status: status.to_string(),
                    ports: ports.to_string(),
                });
            }
        }
    }
    None
}

/// Extrait les derniers logs d'un conteneur via la commande Docker CLI.
fn fetch_container_logs(container_id: &str, tail_lines: usize) -> String {
    let output = Command::new("docker")
        .args(["logs", "--tail", &tail_lines.to_string(), container_id])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            let mut combined = String::new();
            if !stdout.trim().is_empty() {
                combined.push_str(&stdout.trim());
            }
            if !stderr.trim().is_empty() {
                if !combined.is_empty() {
                    combined.push('\n');
                }
                combined.push_str(stderr.trim());
            }

            if combined.is_empty() {
                "Aucun log récent disponible.".to_string()
            } else {
                combined
            }
        }
        Err(err) => format!("Erreur d'exécution de la commande docker logs: {}", err),
    }
}