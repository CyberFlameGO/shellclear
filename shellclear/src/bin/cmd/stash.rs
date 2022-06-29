use anyhow::Result;
use clap::{crate_name, ArgMatches, Command};
use shellclear::{promter::confirm, ShellContext};

pub fn command() -> Command<'static> {
    Command::new("stash")
        .about("Stash history changes and stash the commands")
        .subcommand(Command::new("pop").about("Pop stash history commands"))
}

pub fn run(
    subcommand_matches: &ArgMatches,
    shell_context: &ShellContext,
) -> Result<shellclear::CmdExit> {
    match subcommand_matches.subcommand() {
        None => run_stash(shell_context),
        Some(tup) => match tup {
            ("pop", _subcommand_matches) => run_pop(shell_context),
            _ => unreachable!(),
        },
    }
}

fn run_stash(shell_context: &ShellContext) -> Result<shellclear::CmdExit> {
    // todo:: check if file exists and remove the unwrap
    if shell_context.is_stash_file_exists()? {
        if let Err(e) = confirm("Stash file already find. do you want to override? (you can lose all your history commands)"){
            log::debug!("{:?}", e);
            return Ok(shellclear::CmdExit {
                code: exitcode::OK,
                message: None,
            });
        }
    }

    if let Err(err) = shell_context.stash() {
        return Ok(shellclear::CmdExit {
            code: 1,
            message: Some(format!("stash failed: {:?}", err)),
        });
    }
    Ok(shellclear::CmdExit {
        code: 0,
        message: Some(format!("Shell {:?} stash successfully when opening a new tab. Run `{} stash pop` to return your history commands",shell_context.history.shell, crate_name!() )),
    })
}

fn run_pop(shell_context: &ShellContext) -> Result<shellclear::CmdExit> {
    if !shell_context.is_stash_file_exists()? {
        return Ok(shellclear::CmdExit {
            code: 1,
            message: Some("Stash file not found".to_string()),
        });
    }

    if let Err(err) = shell_context.pop() {
        return Ok(shellclear::CmdExit {
            code: 1,
            message: Some(format!("stash pop failed: {:?}", err)),
        });
    }
    Ok(shellclear::CmdExit {
        code: 0,
        message: Some(format!(
            "Shell {:?} history pop successfully when open a new tab. ",
            shell_context.history.shell
        )),
    })
}
