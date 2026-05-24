// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::{Parser, Subcommand};

mod audit_log;
mod daemon_client;
mod diff;
mod gates;
mod pipeline;
mod preview_cmd;
mod process;
mod provenance;
mod publish;
mod secret_scanner;
mod template_engine;
mod template_lock;

#[derive(Parser)]
#[command(name = "sbfb-factory", about = "SBFB app scaffolding tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new SBFB app from a template
    Create {
        /// Template to use
        #[arg(long, default_value = "static")]
        template: String,

        /// App name
        #[arg(long)]
        name: String,

        /// Output directory (defaults to ./<name>)
        #[arg(long)]
        output: Option<String>,
    },

    /// Validate an existing SBFB project
    Validate {
        /// Path to the project directory
        path: String,
    },

    /// Load an ephemeral preview into the local daemon
    Preview {
        /// Path to the project directory
        path: String,
    },

    /// Publish a project from its source repository
    Publish {
        /// Path to the project directory
        path: String,

        /// Public repository URL (HTTPS)
        #[arg(long)]
        repo_url: String,

        /// Skip pre-publish gates (FG4/FG5/FG6) for debugging
        #[arg(long, default_value_t = false)]
        skip_gates: bool,
    },

    /// Show diff between workspace and template
    Diff {
        /// Path to the project directory
        path: String,
    },

    /// Scan a project directory for secrets
    ScanSecrets {
        /// Path to the project directory
        path: String,
    },

    /// Run FG5 sandbox check (symlinks, path traversal)
    Sandbox {
        /// Path to the project directory
        path: String,
    },

    /// Run FG7 preview readiness check (daemon connectivity)
    PreviewCheck {
        /// Path to the project directory
        path: String,
    },

    /// Process observability and prompt assembly
    Process {
        #[command(subcommand)]
        subcmd: ProcessCommand,
    },
}

#[derive(Subcommand)]
enum ProcessCommand {
    /// Show repo context (sprint, phase, HEAD, artifacts, AGENT_SYSTEM)
    Context,

    /// Assemble a portable prompt by kind
    Prompt {
        /// Prompt kind: handoff, preflight, phase-review, commit-body, audit-gate, phase-auditor
        #[arg(long)]
        kind: String,

        /// Output depth: standard or deep
        #[arg(long, default_value = "standard")]
        depth: String,

        /// Target provider: claude, codex, gpt, local, human
        #[arg(long, default_value = "claude")]
        provider: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let (cmd_name, cmd_args, result): (&str, Vec<String>, Result<(), Box<dyn std::error::Error>>) =
        match cli.command {
            Command::Create {
                template,
                name,
                output,
            } => {
                let args = vec![format!("--template={template}"), format!("--name={name}")];
                let output_dir = output.unwrap_or_else(|| name.clone());
                let r =
                    template_engine::create(&template, &name, &output_dir).map_err(|e| e.into());
                ("create", args, r)
            }
            Command::Validate { path } => {
                let args = vec![path.clone()];
                let r = template_engine::validate(&path).map_err(|e| e.into());
                ("validate", args, r)
            }
            Command::Preview { path } => {
                let args = vec![path.clone()];
                let r = preview_cmd::run(&path);
                ("preview", args, r)
            }
            Command::Publish {
                path,
                repo_url,
                skip_gates,
            } => {
                let args = vec![path.clone(), format!("--repo-url={repo_url}")];
                let r = publish::run(&path, &repo_url, skip_gates);
                ("publish", args, r)
            }
            Command::Diff { path } => {
                let args = vec![path.clone()];
                let r = run_diff(&path);
                ("diff", args, r)
            }
            Command::ScanSecrets { path } => {
                let args = vec![path.clone()];
                let r = run_scan_secrets(&path);
                ("scan-secrets", args, r)
            }
            Command::Sandbox { path } => {
                let args = vec![path.clone()];
                let r = run_sandbox(&path);
                ("sandbox", args, r)
            }
            Command::PreviewCheck { path } => {
                let args = vec![path.clone()];
                let r = run_preview_check(&path);
                ("preview-check", args, r)
            }
            Command::Process { subcmd } => match subcmd {
                ProcessCommand::Context => {
                    let r = process::run_context();
                    ("process-context", vec![], r)
                }
                ProcessCommand::Prompt {
                    kind,
                    depth,
                    provider,
                } => {
                    let args = vec![
                        format!("--kind={kind}"),
                        format!("--depth={depth}"),
                        format!("--provider={provider}"),
                    ];
                    let r = process::run_prompt(&kind, &depth, &provider);
                    ("process-prompt", args, r)
                }
            },
        };

    let result_str = match &result {
        Ok(()) => "success".to_string(),
        Err(e) => format!("error: {e}"),
    };

    let entry = audit_log::AuditEntry {
        timestamp: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        command: cmd_name.to_string(),
        args: cmd_args,
        result: result_str,
    };
    let _ = audit_log::log_entry(&entry);

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run_diff(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = dunce::canonicalize(path)?;
    let result = gates::run_gate_fg4_diff(&workspace)?;
    eprintln!("{result}");
    Ok(())
}

fn run_scan_secrets(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = dunce::canonicalize(path)?;
    let result = gates::run_gate_fg6_secrets(&workspace)?;
    eprintln!("{result}");
    if !result.passed {
        return Err("secrets detected in project".into());
    }
    Ok(())
}

fn run_sandbox(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = dunce::canonicalize(path)?;
    let result = gates::run_gate_fg5_sandbox(&workspace)?;
    eprintln!("{result}");
    if !result.passed {
        return Err("sandbox check failed".into());
    }
    let index = workspace.join("index.html");
    if index.exists() {
        let contained = gates::check_path_containment(&workspace, &index)?;
        if !contained {
            return Err("index.html escapes workspace".into());
        }
    }
    Ok(())
}

fn run_preview_check(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = dunce::canonicalize(path)?;
    let result = gates::run_gate_fg7_preview(&workspace)?;
    eprintln!("{result}");
    if !result.passed {
        return Err("preview check failed".into());
    }
    Ok(())
}
