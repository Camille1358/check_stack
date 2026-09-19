use colored::*;
use reqwest::blocking::Client;
use std::time::{Duration, Instant};

struct ServiceTest {
    name: &'static str,
    url: &'static str,
    port: u16,
}

struct TestResult {
    name: &'static str,
    port: u16,
    status: &'static str,
    latency_ms: u128,
    details: String,
}

fn main() {
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{}", "                      INSPECTION GLOBALE DE LA STACK IA LOCAL                     ".bold().cyan());
    println!("{}", "==========================================================================================".bold().cyan());
    println!();

    let client = Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .expect("Échec de création du client HTTP");

    let services = vec![
        ServiceTest { name: "Ollama (N1.1)", url: "http://127.0.0.1:11434/api/version", port: 11434 },
        ServiceTest { name: "Mistral.rs (N1.2)", url: "http://127.0.0.1:1234/v1/models", port: 1234 },
        ServiceTest { name: "Llama.cpp (N1.3)", url: "http://127.0.0.1:8085/v1/models", port: 8085 },
        ServiceTest { name: "Qdrant Vector DB (N2.1)", url: "http://127.0.0.1:6333/healthz", port: 6333 },
        ServiceTest { name: "TEI Embeddings (N2.3)", url: "http://127.0.0.1:8080/health", port: 8080 },
        ServiceTest { name: "Mem0 Memory (N2.4)", url: "http://127.0.0.1:8081/health", port: 8081 },
        ServiceTest { name: "LiteLLM Proxy (N3.2)", url: "http://127.0.0.1:4000/health/readiness", port: 4000 },
        ServiceTest { name: "Guardrails AI (N4.2)", url: "http://127.0.0.1:8005/health", port: 8005 },
        ServiceTest { name: "OmniRoute Façade (N3.1)", url: "http://127.0.0.1:3000/v1/models", port: 3000 },
        ServiceTest { name: "Langfuse Tracing (N5.4)", url: "http://127.0.0.1:3001/api/public/health", port: 3001 },
        ServiceTest { name: "SearXNG Search (N6.3)", url: "http://127.0.0.1:8888/search?q=test&format=json", port: 8888 },
        ServiceTest { name: "Open WebUI (N7.1)", url: "http://127.0.0.1:8082", port: 8082 },
        ServiceTest { name: "n8n Workflows (N7.4)", url: "http://127.0.0.1:5678", port: 5678 },
    ];

    let mut results = Vec::new();

    println!("{}", "[LOGS DETAILLES DE NAVIGATION ET REPONSES]".bold().yellow());
    println!("{}", "------------------------------------------------------------------------------------------".dimmed());

    for service in services {
        print!("[TEST] Interrogation de {:<22} ({:<38}) ... ", service.name.bold(), service.url);
        let start = Instant::now();
        let response = client.get(service.url).send();
        let duration = start.elapsed().as_millis();

        match response {
            Ok(resp) => {
                let code = resp.status();
                if code.is_success() || code.is_redirection() {
                    println!("{}", format!("SUCCES ({}) - {}ms", code, duration).green());
                    results.push(TestResult {
                        name: service.name,
                        port: service.port,
                        status: "ACTIF",
                        latency_ms: duration,
                        details: format!("HTTP {}", code),
                    });
                } else {
                    println!("{}", format!("REPONSE ANORMALE ({}) - {}ms", code, duration).yellow());
                    results.push(TestResult {
                        name: service.name,
                        port: service.port,
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
                    status: "INACTIF",
                    latency_ms: 0,
                    details: err_msg.to_string(),
                });
            }
        }
    }

    println!("\n\n");
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{}", "                                 TABLEAU RECAPITULATIF                                   ".bold().cyan());
    println!("{}", "==========================================================================================".bold().cyan());
    println!("{:<26} | {:<7} | {:<10} | {:<10} | {:<25}", "SERVICE", "PORT", "STATUT", "LATENCE", "DETAILS");
    println!("{}", "------------------------------------------------------------------------------------------".cyan());

    for res in results {
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
}