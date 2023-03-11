use clap::{arg, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(long, default_value = "dev")]
    pub env: String,
    #[clap(long, default_value_t = false, env = "IS_LOCAL_RUN", action = clap::ArgAction::Set)]
    pub is_local_run: bool,
    #[command(subcommand)]
    pub processor: Processors,
}

#[derive(Subcommand, Debug)]
pub enum Processors {
    TestProcess(TestProcess),
    TestGenerate(TestGenerate),
    TestDBProcess(TestDBProcess),
    TestRequestProcess,
}

#[derive(Args, Debug)]
pub struct TestProcess {
    #[arg(long, default_value_t = 50)]
    pub wait_ms: u64,
}

#[derive(Args, Debug)]
pub struct TestGenerate {
    #[arg(long, default_value_t = 50)]
    pub wait_ms: u64,
}

#[derive(Args, Debug)]
pub struct TestDBProcess {
    #[arg(long, default_value_t = 50)]
    pub wait_ms: u64,
}
