#[macro_use]
extern crate execute_command_macro;

use colored::Colorize;
use spinoff::{spinners, Color, Spinner};
use std::{ffi::OsStr, process::Command};
use which::which;

pub enum Install {
    Cmd(Command),
    Url(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum LocateStatus {
    Abort(&'static str),
    Found,
    NotFound,
}

struct Dependency {
    name: &'static str,
    locate: Box<dyn Fn() -> LocateStatus>,
    install: Vec<Install>,
}

impl Dependency {
    pub fn locate(&self) -> LocateStatus {
        (self.locate)()
    }
}

fn print_err(out: &str) {
    let bar = "|".red();
    eprintln!("{bar}");
    for line in out.lines() {
        eprintln!("{bar:<2}{line}");
    }
    eprintln!("{bar}");
}

fn exists<S: AsRef<OsStr>>(executable: S) -> LocateStatus {
    let exe = executable.as_ref();
    which(exe)
        .map(|_| LocateStatus::Found)
        .unwrap_or(LocateStatus::NotFound)
}

#[derive(Clone, Debug)]
enum InstallStatus {
    Failed(String),
    Ok,
}

fn install(mut spinner: Spinner, mut cmd: Command, dep: &str) -> InstallStatus {
    fn fail(spinner: Spinner, dep: &str) {
        spinner.fail(&format!("Install  {:.<20}{}", dep, "failed".red()));
    }

    spinner.update_text(format!("Installing {:.<20}", dep));
    match cmd.output() {
        Ok(out) => {
            if out.status.success() {
                spinner.success(&format!("Install  {:.<20}{}", dep, "ok".green()));
                InstallStatus::Ok
            } else {
                fail(spinner, dep);
                let stderr = String::from_utf8_lossy(&out.stderr);
                print_err(&stderr);
                InstallStatus::Failed(format!("Failed to install {dep}"))
            }
        }
        Err(e) => {
            fail(spinner, dep);
            let msg = format!("Command: {cmd:?}\nError: {e}");
            print_err(&msg);
            InstallStatus::Failed(format!("Failed to install {dep}"))
        }
    }
}

mod exe {
    pub const PSQL: &str = "psql";
    pub const RUSTUP: &str = "rustup";
    pub const NPX: &str = "npx";
    pub const WATCHEXEC: &str = "watchexec";
    pub const NEXTEST: &str = "nextest";
}

fn main() {
    let dependencies = vec![
        Dependency {
            name: exe::NPX,
            locate: Box::new(|| exists(exe::NPX)),
            install: vec![Install::Url(
                "https://docs.npmjs.com/downloading-and-installing-node-js-and-npm",
            )],
        },
        Dependency {
            name: exe::PSQL,
            locate: Box::new(|| exists(exe::PSQL)),
            install: vec![Install::Url("https://www.postgresql.org/download/")],
        },
        Dependency {
            name: exe::RUSTUP,
            locate: Box::new(|| {
                which(exe::RUSTUP)
                    .map(|_| LocateStatus::Found)
                    .unwrap_or(LocateStatus::Abort(
                    "Installation of 'rustup' is required before checking remaining dependencies",
                ))
            }),
            install: vec![Install::Url("https://rustup.rs/")],
        },
        Dependency {
            name: "rust wasm32 target",
            locate: Box::new(|| {
                command!("rustup target list")
                    .output()
                    .map(|out| {
                        if String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .any(|line| line == "wasm32-unknown-unknown (installed)")
                        {
                            LocateStatus::Found
                        } else {
                            LocateStatus::NotFound
                        }
                    })
                    .unwrap_or(LocateStatus::NotFound)
            }),
            install: vec![Install::Cmd(command!(
                "rustup target add wasm32-unknown-unknown"
            ))],
        },
        Dependency {
            name: exe::WATCHEXEC,
            locate: Box::new(|| exists("watchexec")),
            install: vec![Install::Cmd(command!("cargo install --locked watchexec-cli"))],
        },
        Dependency {
            name: exe::NEXTEST,
            locate: Box::new(|| exists("cargo-nextest")),
            install: vec![Install::Cmd(command!("cargo install cargo-nextest --locked"))],
        },
    ];

    let mut manual_install = Vec::new();
    let mut errors = Vec::new();

    'depcheck: for dep in dependencies {
        print!("  Checking {:.<20}", dep.name);
        match dep.locate() {
            LocateStatus::Abort(msg) => {
                println!("{}", "abort".red());
                let msg = msg.red();
                eprintln!("  {msg}");
                if let Some(Install::Url(url)) = dep.install.first() {
                    manual_install.push((dep.name, *url));
                }
                break 'depcheck;
            }
            LocateStatus::Found => println!("{}", "ok".green()),
            LocateStatus::NotFound => {
                println!("{}", "not found".red());
                for instruction in dep.install {
                    let spinner = Spinner::new(spinners::Dots, "", Color::Blue);
                    match instruction {
                        Install::Cmd(cmd) => match install(spinner, cmd, dep.name) {
                            InstallStatus::Ok => (),
                            InstallStatus::Failed(msg) => errors.push(msg),
                        },
                        Install::Url(url) => {
                            spinner.warn(&format!(
                                "Install  {:.<20}{}",
                                dep.name,
                                "manual".yellow()
                            ));
                            manual_install.push((dep.name, url));
                        }
                    }
                }
            }
        }
    }
}
