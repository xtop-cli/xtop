//! Monitoring, diagnostics and shell-integration help text.

pub(crate) const MCP: &str = "\
xtop mcp - run the MCP server over stdio

Usage:
  xtop mcp

Starts the MCP extension so AI agents can call the samurai plugin tools.
Requires the plugin-samurai and mcp-extension features (enabled by default).

Options:
  -h, --help    Show this help and exit";

pub(crate) const STATUS: &str = "\
xtop status - one-shot system snapshot

Usage:
  xtop status [--json]

Samples CPU, memory, swap, load, uptime, disks, network, batteries and GPUs.
The command samples twice with a short pause so CPU and rate values are
meaningful on the first output.

Options:
  --json        Print a machine-readable JSON object
  -h, --help    Show this help and exit";

pub(crate) const PROCESSES: &str = "\
xtop processes - one-shot process list

Usage:
  xtop processes [options]

Options:
  --json              Print a machine-readable JSON object
  --sort <column>     cpu (default), memory (mem), pid or name
  --asc               Sort ascending
  --desc              Sort descending (default)
  --limit <n>         Maximum rows; 0 means all (default 20)
  --filter <text>     Only process names containing text
  -h, --help          Show this help and exit";

pub(crate) const WATCH: &str = "\
xtop watch - periodic system snapshots

Usage:
  xtop watch [options]

Options:
  --json, --jsonl     Print one compact JSON object per sample
  --interval <ms>     Sampling interval (default 1000, minimum 100)
  --count <n>         Stop after n samples; 0 means run until Ctrl-C
  -h, --help          Show this help and exit";

pub(crate) const INFO: &str = "\
xtop info - show build, path and tool information

Usage:
  xtop info [--json]

Prints the version, target, enabled features, configuration/asset paths,
local asset-file counts and whether curl, git and cargo are available.

Options:
  --json        Print a machine-readable JSON object
  -h, --help    Show this help and exit";

pub(crate) const DOCTOR: &str = "\
xtop doctor - diagnose the local installation

Usage:
  xtop doctor [--json]

Checks config-directory writability, config.json and assets.json parsing,
base theme/layout presence, the catalog URL and the external tools used by
downloads and developer commands. Exits with status 1 only when a check
fails; warnings do not fail.

Options:
  --json        Print a machine-readable JSON object
  -h, --help    Show this help and exit";

pub(crate) const COMPLETIONS: &str = "\
xtop completions - print shell completions

Usage:
  xtop completions <bash|zsh|fish|powershell|pwsh>

Writes the completion script to stdout.

Options:
  -h, --help    Show this help and exit";

pub(crate) const UPDATE: &str = "\
xtop update - update the running installation

Usage:
  xtop update [options]

Detects the installation medium and uses the same one:
  xpm     installed by the X package manager
  pacman  installed by pacman
  cargo   installed under the cargo bin directory or via cargo install
  binary  standalone release binary; uses the official prebuilt installer

Options:
  --check              Print the installed/latest versions and the command
  --dry-run            Detect the medium and print the command without running
  --method <method>    Force auto, xpm, pacman, cargo, binary (bin)
  --yes, -y            Non-interactive package-manager confirmation and sudo -n
  --force              Reinstall or force a cargo update
  --all-features       Use --all-features for a cargo installation
  --version <version>  Target a release version (cargo and binary)
  --json               Print a machine-readable result
  -h, --help           Show this help and exit";
