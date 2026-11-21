//! Shell completion scripts for the hand-written CLI.

use crate::cli::help;

pub(crate) fn completions_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::COMPLETIONS);
        return Ok(());
    }
    let Some(shell) = args.get(2) else {
        help::usage_error(
            help::COMPLETIONS,
            "Usage: xtop completions <bash|zsh|fish|powershell>",
        );
    };
    match shell.as_str() {
        "bash" => print!("{BASH}"),
        "zsh" => print!("{ZSH}"),
        "fish" => print!("{FISH}"),
        "powershell" | "pwsh" => print!("{POWERSHELL}"),
        other => help::usage_error(
            help::COMPLETIONS,
            &format!("unsupported shell '{other}': use bash, zsh, fish or powershell"),
        ),
    }
    Ok(())
}

const BASH: &str = r#"# xtop bash completion
_xtop() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    case "$prev" in
        theme)  COMPREPLY=( $(compgen -W "list available current show set download path" -- "$cur") ); return ;;
        layout) COMPREPLY=( $(compgen -W "list available current show set download install check path" -- "$cur") ); return ;;
        addon) COMPREPLY=( $(compgen -W "list available install download path" -- "$cur") ); return ;;
        config) COMPREPLY=( $(compgen -W "download generate get set list reset path" -- "$cur") ); return ;;
        plugin) COMPREPLY=( $(compgen -W "list install scaffold" -- "$cur") ); return ;;
        widget) COMPREPLY=( $(compgen -W "list install scaffold" -- "$cur") ); return ;;
        completions) COMPREPLY=( $(compgen -W "bash zsh fish powershell" -- "$cur") ); return ;;
    esac
    COMPREPLY=( $(compgen -W "status processes watch info doctor completions addon theme layout config plugin widget mcp --help --version --change-theme --gen-config" -- "$cur") )
}
complete -F _xtop xtop
"#;

const ZSH: &str = r#"#compdef xtop
# xtop zsh completion
_xtop() {
    local -a commands
    commands=(
        'status:one-shot system snapshot'
        'processes:one-shot process list'
        'watch:periodic system snapshots'
        'info:build, path and tool information'
        'doctor:diagnose the local installation'
        'completions:print shell completions'
        'theme:manage themes'
        'layout:manage layouts'
        'config:manage config and assets'
        'plugin:manage compiled-in plugins'
        'widget:manage compiled-in widget packs'
        'mcp:run the MCP server'
        '--change-theme:change the active theme'
        '--gen-config:create the base config and assets'
        '--version:print the version'
        '--help:print help'
    )
    _describe 'command' commands
}
_xtop "$@"
"#;

const FISH: &str = r#"# xtop fish completion
complete -c xtop -f -n '__fish_use_subcommand' -a 'status processes watch info doctor completions addon theme layout config plugin widget mcp'
complete -c xtop -f -n '__fish_use_subcommand' -a '--change-theme --gen-config --version --help'
complete -c xtop -f -n '__fish_seen_subcommand_from theme' -a 'list available current show set download path'
complete -c xtop -f -n '__fish_seen_subcommand_from layout' -a 'list available current show set download install check path'
complete -c xtop -f -n '__fish_seen_subcommand_from addon' -a 'list available install download path'
complete -c xtop -f -n '__fish_seen_subcommand_from config' -a 'download generate get set list reset path'
complete -c xtop -f -n '__fish_seen_subcommand_from plugin' -a 'list install scaffold'
complete -c xtop -f -n '__fish_seen_subcommand_from widget' -a 'list install scaffold'
complete -c xtop -f -n '__fish_seen_subcommand_from completions' -a 'bash zsh fish powershell'
"#;

const POWERSHELL: &str = r#"# xtop PowerShell completion
Register-ArgumentCompleter -Native -CommandName xtop -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = 'status','processes','watch','info','doctor','completions','theme','layout','config','plugin','widget','mcp','--change-theme','--gen-config','--version','--help'
    $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#;
