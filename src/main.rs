use std::{path::Path, process::ExitStatus, time::Duration};

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecursiveMode, Watcher},
};
use tokio::{fs::File, process::Command, select};
use typstpp::{preprocess_typst, Error};

#[derive(Debug, Parser)]
#[clap(name = "typstpp", version, author, about)]
struct CliArgs {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    #[clap(about = "Print typstpp info")]
    Info,
    #[clap(about = "Preprocess a typst file")]
    Preprocess(PreprocessArgs),
    #[clap(about = "Preprocess and compile a typst file")]
    Compile(CompileArgs),
    #[clap(about = "Watch a typst file and preprocess then recompile on changes")]
    Watch(WatchArgs),
}

#[derive(Debug, Parser)]
struct PreprocessArgs {
    #[clap(short, long)]
    input: String,
    #[clap(short, long)]
    output: Option<String>,
}

#[derive(Debug, Parser)]
struct CompileArgs {
    #[clap(short, long)]
    input: String,
    #[clap(short, long)]
    output: Option<String>,

    #[clap(last = true)]
    typst_args: Vec<String>,
}

#[derive(Debug, Parser)]
struct WatchArgs {
    #[clap(short, long)]
    input: String,
    #[clap(short, long)]
    output: Option<String>,

    #[clap(last = true)]
    typst_args: Vec<String>,
}

const VERB_WIDTH: usize = 15;

fn pad(s: &str) -> String {
    format!("{:width$}", s, width = VERB_WIDTH)
}

fn log_process(verb: &str, msg: &str) {
    println!("{} {}", pad(verb).cyan(), msg);
}

fn log_success(verb: &str, dur: Duration, msg: &str) {
    println!("{} {} in {}", pad(verb).green(), msg, {
        let secs = dur.as_secs();
        let millis = dur.subsec_millis();
        format!("{}.{:03}s", secs, millis)
    });
}

fn log_err(verb: &str, msg: &str) {
    eprintln!("{} {}", pad(verb).red(), msg);
}

#[derive(Debug, thiserror::Error)]
enum CompileError {
    #[error("process error: {0}")]
    ProcessError(tokio::io::Error),
    #[error("exit status: {0}")]
    ExitStatus(ExitStatus),
}

impl From<tokio::io::Error> for CompileError {
    fn from(e: tokio::io::Error) -> Self {
        CompileError::ProcessError(e)
    }
}

impl From<ExitStatus> for CompileError {
    fn from(e: ExitStatus) -> Self {
        CompileError::ExitStatus(e)
    }
}

async fn compile_typst(
    input: &str,
    output: Option<&str>,
    typst_args: &Vec<String>,
) -> Result<(), CompileError> {
    let mut cmd = Command::new("typst");
    cmd.arg("compile").args(typst_args).arg(input);
    if let Some(output) = output {
        cmd.arg(output);
    }
    let exit = cmd.spawn()?.wait().await?;
    if exit.success() {
        Ok(())
    } else {
        Err(exit.into())
    }
}

async fn compile_typst_and_log(
    input: &str,
    output: Option<&str>,
    typst_args: &Vec<String>,
) -> Result<(), CompileError> {
    if typst_args.is_empty() {
        log_process("Compiling", input);
    } else {
        log_process(
            "Compiling",
            format!("{} with {}", input, typst_args.join(" ")).as_str(),
        );
    }
    let start = std::time::Instant::now();
    match compile_typst(input, output, typst_args).await {
        Ok(_) => {
            log_success("Compiled", start.elapsed(), input);
            Ok(())
        }
        Err(e) => {
            log_err("Failed", input);
            Err(e)
        }
    }
}

async fn preprocess(inputf: &str, output: &str) -> Result<(), Error> {
    let mut input = File::open(inputf).await?;
    let mut output = File::create(output).await?;
    preprocess_typst(&mut input, &mut output).await?;
    Ok(())
}

async fn preprocess_and_log(inputf: &str, output: &str) -> Result<(), Error> {
    log_process("Preprocessing", inputf);
    let start = std::time::Instant::now();
    match preprocess(inputf, output).await {
        Ok(_) => {
            log_success("Preprocessed", start.elapsed(), inputf);
            Ok(())
        }
        Err(e) => {
            log_err("Failed", inputf);
            Err(e)
        }
    }
}

fn infer_preprocess_output<P: AsRef<Path>>(input: P) -> String {
    let input = input.as_ref();
    let mut output = input.file_stem().unwrap().to_os_string();
    output.push(".out.typ");
    output.into_string().unwrap()
}

#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
    match cli.subcmd {
        SubCommand::Info => {
            let mut supported_langs: Vec<&'static str> = Vec::new();
            #[cfg(feature = "r")]
            supported_langs.push("r");
            #[cfg(feature = "hs")]
            supported_langs.push("hs");
            println!("Supported languages: {}", supported_langs.join(" "));
        }
        SubCommand::Preprocess(args) => {
            let output = args
                .output
                .unwrap_or_else(|| infer_preprocess_output(&args.input).as_str().to_string());
            preprocess_and_log(&args.input, &output).await.unwrap();
        }
        SubCommand::Compile(args) => {
            let output = args
                .output
                .unwrap_or_else(|| infer_preprocess_output(&args.input).as_str().to_string());
            preprocess_and_log(&args.input, &output).await.unwrap();
            compile_typst_and_log(&output, None, &args.typst_args)
                .await
                .unwrap();
        }
        SubCommand::Watch(args) => {
            let output = args
                .output
                .unwrap_or_else(|| infer_preprocess_output(&args.input).as_str().to_string());
            preprocess_and_log(&args.input, &output).await.ok();
            compile_typst_and_log(&output, None, &args.typst_args)
                .await
                .ok();

            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            let mut debouncer = new_debouncer(Duration::from_secs(1), None, move |res| match res {
                Ok(event) => {
                    tx.blocking_send(event).unwrap();
                }
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                }
            })
            .unwrap();
            debouncer
                .watcher()
                .watch(Path::new(&args.input), RecursiveMode::NonRecursive)
                .unwrap();

            loop {
                select! {
                        _ = tokio::signal::ctrl_c() => {
                            debouncer.stop();
                            return;
                        }
                        Some(_) = rx.recv() => {
                            preprocess_and_log(&args.input, &output).await.ok();
                            compile_typst_and_log(&output, None, &args.typst_args).await.ok();
                        }
                }
            }
        }
    }
}
