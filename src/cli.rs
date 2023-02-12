use clap::{arg, Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub processor: Processors,
}

#[derive(Subcommand)]
pub enum Processors {
    TestProcess(TestProcess),
    TestGenerate(TestGenerate),
}

#[derive(Args)]
pub struct TestProcess {
    #[arg(long, default_value_t = 50)]
    pub wait_ms: u64,
}

#[derive(Args)]
pub struct TestGenerate {
    #[arg(long, default_value_t = 50)]
    pub wait_ms: u64,
}
