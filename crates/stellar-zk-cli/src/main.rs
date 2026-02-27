//! stellar-zk CLI — the unified ZK development toolkit for Stellar/Soroban.
//!
//! Provides six commands that cover the full ZK development lifecycle:
//! `init`, `build`, `prove`, `deploy`, `call`, and `estimate`.
//!
//! Each command delegates to a backend-specific implementation via the
//! [`stellar_zk_core::backend::ZkBackend`] trait.

mod commands;
mod output;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "stellar-zk",
    about = "ZK DevKit for Stellar/Soroban — Groth16 + UltraHonk + RISC Zero",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to stellar-zk.config.json (default: ./stellar-zk.config.json)
    #[arg(long, global = true, default_value = "stellar-zk.config.json")]
    config: PathBuf,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new ZK project
    Init {
        /// Project name (creates a directory with this name)
        name: String,

        /// ZK backend to use
        #[arg(long, value_enum)]
        backend: Option<BackendChoice>,

        /// Optimization profile
        #[arg(long, default_value = "development")]
        profile: ProfileChoice,
    },

    /// Build the ZK circuit/program and Soroban verifier contract
    Build {
        /// Override the optimization profile
        #[arg(long)]
        profile: Option<ProfileChoice>,

        /// Only build the circuit (skip contract)
        #[arg(long)]
        circuit_only: bool,

        /// Only build the contract (skip circuit)
        #[arg(long)]
        contract_only: bool,
    },

    /// Generate a ZK proof
    Prove {
        /// Path to input JSON file
        #[arg(long, short)]
        input: PathBuf,

        /// Output path for the proof file
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Deploy the verifier contract to Stellar
    Deploy {
        /// Stellar network
        #[arg(long, default_value = "testnet")]
        network: NetworkChoice,

        /// Source account secret key or identity name
        #[arg(long)]
        source: String,
    },

    /// Call the deployed contract with a proof
    Call {
        /// Contract ID (C...) on the network
        #[arg(long)]
        contract_id: String,

        /// Path to proof file
        #[arg(long)]
        proof: PathBuf,

        /// Path to public inputs file
        #[arg(long)]
        public_inputs: Option<PathBuf>,

        /// Stellar network
        #[arg(long, default_value = "testnet")]
        network: NetworkChoice,

        /// Source account
        #[arg(long)]
        source: String,
    },

    /// Estimate execution costs for on-chain verification
    Estimate {
        /// Path to proof file (generates a static estimate if omitted)
        #[arg(long)]
        proof: Option<PathBuf>,

        /// Number of public inputs (for static estimation)
        #[arg(long, default_value = "2")]
        public_inputs: u32,

        /// Stellar network to simulate against
        #[arg(long, default_value = "testnet")]
        network: NetworkChoice,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum BackendChoice {
    Groth16,
    Ultrahonk,
    Risc0,
}

impl BackendChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Groth16 => "groth16",
            Self::Ultrahonk => "ultrahonk",
            Self::Risc0 => "risc0",
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProfileChoice {
    Development,
    Testnet,
    StellarProduction,
}

impl ProfileChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Testnet => "testnet",
            Self::StellarProduction => "stellar-production",
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum NetworkChoice {
    Local,
    Testnet,
    Mainnet,
}

impl NetworkChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Testnet => "testnet",
            Self::Mainnet => "mainnet",
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Init {
            name,
            backend,
            profile,
        } => {
            commands::init::run(&name, backend, &profile).await?;
        }
        Commands::Build {
            profile,
            circuit_only,
            contract_only,
        } => {
            commands::build::run(&cli.config, profile, circuit_only, contract_only).await?;
        }
        Commands::Prove { input, output } => {
            commands::prove::run(&cli.config, &input, output.as_deref()).await?;
        }
        Commands::Deploy { network, source } => {
            commands::deploy::run(&cli.config, &network, &source).await?;
        }
        Commands::Call {
            contract_id,
            proof,
            public_inputs,
            network,
            source,
        } => {
            commands::call::run(
                &cli.config,
                &contract_id,
                &proof,
                public_inputs.as_deref(),
                &network,
                &source,
            )
            .await?;
        }
        Commands::Estimate {
            proof,
            public_inputs,
            network,
        } => {
            commands::estimate::run(
                &cli.config,
                proof.as_deref(),
                public_inputs,
                &network,
            )
            .await?;
        }
    }

    Ok(())
}
