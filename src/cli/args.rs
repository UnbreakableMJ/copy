use crate::config::config_command::ConfigCommand;
use crate::config::loader::{load_config, load_config_file};
use crate::config::schema::Config;
use crate::error::{CliError, CliResult};
use crate::utility::helper::parse_progress_bar;
use crate::utility::progress_bar::ProgressOptions;
use crate::utility::{
    exclude::{ExcludePattern, ExcludeRules, build_exclude_rules, parse_exclude_pattern_list},
    helper::{parse_backup_mode, parse_follow_symlink, parse_reflink_mode, parse_symlink_mode},
    preserve::PreserveAttr,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SymlinkMode {
    Auto,
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum ReflinkMode {
    Always,
    Auto,
    Clone,
    Copy,
    Never,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum BackupMode {
    None,
    Numbered,
    Existing,
    Simple,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FollowSymlink {
    NoDereference,
    Dereference,
    CommandLineSymlink,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

/// Dedicated parser for the `config` subcommand.
///
/// Copy is the root action, so its positional `sources` are a greedy variadic
/// that would otherwise swallow `config show` as two source paths. When the
/// first argument is `config` we parse it through this grammar instead, which
/// has no copy positionals to compete with the subcommand.
#[derive(Parser, Debug)]
#[command(name = "copy")]
struct ConfigInvocation {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
#[command(
    name = "copy",
    version = env!("CARGO_PKG_VERSION"),
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct CLIArgs {
    // Input/Output Options
    #[arg(help = "Source file(s) or directory(ies)", required = true)]
    pub sources: Vec<PathBuf>,

    #[arg(help = "Destination file or directory", required = true)]
    pub destination: PathBuf,

    #[arg(
        short = 't',
        long = "target-directory",
        value_name = "DIRECTORY",
        help = "copy all SOURCE arguments into DIRECTORY"
    )]
    pub target_directory: Option<PathBuf>,

    #[arg(
        short = 'e',
        long = "exclude",
        value_name = "PATTERN",
        help = "Exclude files matching pattern (can be specified multiple times, supports comma-separated values)"
    )]
    pub exclude: Vec<String>,

    // Copy Behavior Options
    #[arg(short, long, help = "Copy directories recursively")]
    pub recursive: bool,

    #[arg(
        short = 'j',
        default_value_t = 4,
        help = "Number of parallel copy operations for multiple files"
    )]
    pub parallel: usize,

    #[arg(long = "resume", help = "resume interrupted transfers")]
    pub resume: bool,

    #[arg(
        short = 'f',
        long,
        help = "if an existing destination file cannot be opened, remove it and try again"
    )]
    pub force: bool,

    #[arg(short = 'i', long, help = "prompt before overwrite")]
    pub interactive: bool,

    #[arg(long, help = "use full source file name under DIRECTORY")]
    pub parents: bool,

    #[arg(
        long = "attributes-only",
        help = "don't copy the file data, just the attributes"
    )]
    pub attributes_only: bool,

    #[arg(
        long = "remove-destination",
        help = "remove each existing destination file before attempting to open it"
    )]
    pub remove_destination: bool,

    // Link and Symlink Options
    #[arg(
        short = 's',
        long = "symbolic-link",
        value_name = "MODE",
        default_missing_value = "auto",
        num_args = 0..=1,
        help = "make symbolic links instead of copying (auto, absolute, or relative)"
    )]
    pub symbolic_link: Option<SymlinkMode>,

    #[arg(
        short = 'l',
        long = "link",
        help = "hard link files instead of copying"
    )]
    pub hard_link: bool,

    #[arg(
        short = 'P',
        long = "no-dereference",
        help = "never follow symbolic links in SOURCE"
    )]
    pub no_dereference: bool,

    #[arg(
        short = 'L',
        long = "dereference",
        help = "always follow symbolic links in SOURCE"
    )]
    pub dereference: bool,

    #[arg(
        short = 'H',
        long = "dereference-command-line",
        help = "follow symbolic links only on command line"
    )]
    pub dereference_command_line: bool,

    // Preservation Options
    #[arg(
        short = 'p',
        long = "preserve",
        value_name = "ATTR_LIST",
        default_missing_value = "",
        help = "preserve the specified attributes"
    )]
    pub preserve: Option<String>,

    // Backup and Reflink Options
    #[arg(
        short = 'b',
        long = "backup",
        value_name = "CONTROL",
        default_missing_value = "existing",
        num_args = 0..=1,
        help = "make a backup of each existing destination file (none, numbered, existing, simple)"
    )]
    pub backup: Option<BackupMode>,

    #[arg(
        long = "reflink",
        value_name = "WHEN",
        default_missing_value = "auto",
        num_args = 0..=1,
        help = "control clone/CoW copies (auto, always, never)"
    )]
    pub reflink: Option<ReflinkMode>,

    // Config Options (Placed last as meta)
    #[arg(long, value_name = "PATH", help = "Use custom config file")]
    pub config: Option<PathBuf>,

    #[arg(long, help = "Ignore all config files")]
    pub no_config: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone)]
pub struct CopyOptions {
    pub recursive: bool,
    pub parallel: usize,
    pub resume: bool,
    pub force: bool,
    pub interactive: bool,
    pub parents: bool,
    pub preserve: PreserveAttr,
    pub attributes_only: bool,
    pub remove_destination: bool,
    pub symbolic_link: Option<SymlinkMode>,
    pub hard_link: bool,
    pub follow_symlink: FollowSymlink,
    pub progress_bar: ProgressOptions,
    pub backup: Option<BackupMode>,
    pub reflink: Option<ReflinkMode>,
    pub exclude_rules: Option<ExcludeRules>,
    pub abort: Arc<AtomicBool>,
}

impl CopyOptions {
    pub fn none() -> Self {
        Self {
            recursive: false,
            parallel: 4,
            resume: false,
            force: false,
            interactive: false,
            parents: false,
            preserve: PreserveAttr::none(),
            attributes_only: false,
            remove_destination: false,
            symbolic_link: None,
            hard_link: false,
            follow_symlink: FollowSymlink::NoDereference,
            progress_bar: ProgressOptions::default(),
            backup: None,
            reflink: None,
            exclude_rules: None,
            abort: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            recursive: config.copy.recursive,
            parallel: config.copy.parallel,
            resume: config.copy.resume,
            force: config.copy.force,
            interactive: config.copy.interactive,
            parents: config.copy.parents,
            preserve: PreserveAttr::from_string(&config.preserve.mode)
                .unwrap_or_else(|_| PreserveAttr::default()),
            attributes_only: config.copy.attributes_only,
            remove_destination: config.copy.remove_destination,
            symbolic_link: parse_symlink_mode(&config.symlink.mode),
            hard_link: false,
            follow_symlink: parse_follow_symlink(&config.symlink.follow),
            progress_bar: parse_progress_bar(config),
            backup: parse_backup_mode(&config.backup.mode),
            reflink: parse_reflink_mode(&config.reflink.mode),
            exclude_rules: None,
            abort: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl CLIArgs {
    /// Parse command-line arguments.
    ///
    /// Copy is the root action (`copy <src> <dst>`); `config` is the only
    /// subcommand. Because the copy `sources` positional is a greedy variadic,
    /// it would consume `config show` as two source paths, so we special-case a
    /// leading `config` and route it through [`ConfigInvocation`] instead. The
    /// `command` field is still declared on `CLIArgs` so `--help` lists `config`.
    pub fn parse() -> Self {
        let argv: Vec<String> = std::env::args().collect();
        if argv.get(1).map(String::as_str) == Some("config") {
            let ConfigInvocation { command } = <ConfigInvocation as clap::Parser>::parse_from(argv);
            return Self::with_config(command);
        }
        <Self as clap::Parser>::parse_from(argv)
    }

    /// Build a config-mode invocation: copy fields take their defaults and the
    /// parsed `config` subcommand is carried in `command` for `validate` to run.
    fn with_config(command: Commands) -> Self {
        Self {
            sources: Vec::new(),
            destination: PathBuf::new(),
            target_directory: None,
            exclude: Vec::new(),
            recursive: false,
            parallel: 4,
            resume: false,
            force: false,
            interactive: false,
            parents: false,
            attributes_only: false,
            remove_destination: false,
            symbolic_link: None,
            hard_link: false,
            no_dereference: false,
            dereference: false,
            dereference_command_line: false,
            preserve: None,
            backup: None,
            reflink: None,
            config: None,
            no_config: false,
            command: Some(command),
        }
    }

    pub fn validate(self) -> CliResult<(Vec<PathBuf>, PathBuf, CopyOptions)> {
        // Handle the config subcommand (exits the process when present).
        if let Some(Commands::Config { command }) = &self.command {
            command.execute().map_err(|e| {
                CliError::Validation(format!("Failed to execute config command: {}", e))
            })?;
            std::process::exit(0);
        }

        let config = load_config_if_needed(&self).map_err(CliError::Config)?;

        // Start with config or defaults
        let mut options = if let Some(ref cfg) = config {
            CopyOptions::from_config(cfg)
        } else {
            CopyOptions::none()
        };

        // CLI args override config
        apply_cli_overrides(&mut options, &self).map_err(CliError::Validation)?;

        // Build exclude rules
        let all_patterns =
            build_all_exclude_patterns(&self, config.as_ref()).map_err(CliError::Exclude)?;
        options.exclude_rules = build_exclude_rules(all_patterns).map_err(CliError::Exclude)?;

        // Validate conflicts
        validate_conflicts(&options).map_err(CliError::Validation)?;

        // Handle attributes_only special case
        if options.attributes_only {
            options.preserve = PreserveAttr::all();
        }

        let (sources, destination) = if let Some(target) = self.target_directory {
            let mut sources = self.sources;
            sources.push(self.destination);
            (sources, target)
        } else {
            (self.sources, self.destination)
        };

        Ok((sources, destination, options))
    }
}

fn load_config_if_needed(cli: &CLIArgs) -> crate::error::ConfigResult<Option<Config>> {
    if cli.no_config {
        return Ok(None);
    }

    if let Some(custom_path) = &cli.config {
        return Ok(Some(load_config_file(custom_path)?));
    }

    Ok(Some(load_config()))
}

fn apply_cli_overrides(options: &mut CopyOptions, copy_args: &CLIArgs) -> Result<(), String> {
    // Boolean flags - when present, they override
    if copy_args.recursive {
        options.recursive = true;
    }
    if copy_args.force {
        options.force = true;
    }
    if copy_args.interactive {
        options.interactive = true;
    }
    if copy_args.resume {
        options.resume = true;
    }
    if copy_args.parents {
        options.parents = true;
    }
    if copy_args.attributes_only {
        options.attributes_only = true;
    }
    if copy_args.remove_destination {
        options.remove_destination = true;
    }
    if copy_args.hard_link {
        options.hard_link = true;
    }

    // Optional fields - when Some, they override
    if copy_args.symbolic_link.is_some() {
        options.symbolic_link = copy_args.symbolic_link;
    }
    if copy_args.backup.is_some() {
        options.backup = copy_args.backup;
    }
    if copy_args.reflink.is_some() {
        options.reflink = copy_args.reflink;
    }
    if let Some(preserve_str) = &copy_args.preserve {
        options.preserve = PreserveAttr::from_string(preserve_str)
            .map_err(|e| format!("unable to parse preserve attribute: {}", e))?;
    }

    options.parallel = copy_args.parallel;

    options.follow_symlink = copy_args.follow_symlink_mode()?;

    Ok(())
}

fn build_all_exclude_patterns(
    copy_args: &CLIArgs,
    config: Option<&Config>,
) -> crate::error::ExcludeResult<Vec<ExcludePattern>> {
    let mut all_patterns = Vec::new();

    if let Some(cfg) = config {
        for pattern_str in &cfg.exclude.patterns {
            all_patterns.extend(parse_exclude_pattern_list(pattern_str)?);
        }
    }

    all_patterns.extend(copy_args.parse_exclude_patterns()?);
    Ok(all_patterns)
}

fn validate_conflicts(options: &CopyOptions) -> Result<(), String> {
    if options.reflink.is_some() {
        if options.hard_link {
            return Err("--reflink and --link cannot be used together".to_string());
        }
        if options.symbolic_link.is_some() {
            return Err("--reflink and --symbolic-link cannot be used together".to_string());
        }
    }

    if options.symbolic_link.is_some() {
        if options.hard_link {
            return Err("--symbolic-link and --link cannot be used together".to_string());
        }
        if options.resume {
            return Err("--symbolic-link and --continue cannot be used together".to_string());
        }
        if options.attributes_only {
            return Err(
                "--symbolic-link and --attributes-only cannot be used together".to_string(),
            );
        }
    }

    if options.hard_link {
        if options.resume {
            return Err("--link and --continue cannot be used together".to_string());
        }
        if options.attributes_only {
            return Err("--link and --attributes-only cannot be used together".to_string());
        }
    }

    Ok(())
}

impl CLIArgs {
    pub fn follow_symlink_mode(&self) -> Result<FollowSymlink, String> {
        match (
            self.no_dereference,
            self.dereference,
            self.dereference_command_line,
        ) {
            (true, false, false) => Ok(FollowSymlink::NoDereference),
            (false, true, false) => Ok(FollowSymlink::Dereference),
            (false, false, true) => Ok(FollowSymlink::CommandLineSymlink),
            (false, false, false) => Ok(FollowSymlink::NoDereference),
            _ => Err("only one of -P, -L, or -H may be specified".to_string()),
        }
    }

    pub fn parse_exclude_patterns(&self) -> crate::error::ExcludeResult<Vec<ExcludePattern>> {
        let mut patterns = Vec::new();

        for pattern_str in &self.exclude {
            patterns.extend(parse_exclude_pattern_list(pattern_str)?);
        }

        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_symlink_and_hardlink_conflict() {
        let args = CLIArgs {
            sources: vec![PathBuf::from("source.txt")],
            destination: PathBuf::from("dest.txt"),
            target_directory: None,
            recursive: false,
            parallel: 4,
            resume: false,
            force: false,
            interactive: false,
            parents: false,
            preserve: None,
            attributes_only: false,
            remove_destination: false,
            symbolic_link: Some(SymlinkMode::Auto),
            hard_link: true,
            dereference: true,
            no_dereference: false,
            dereference_command_line: false,
            backup: None,
            reflink: None,
            exclude: Vec::new(),
            no_config: false,
            config: None,
            command: None,
        };

        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("symbolic-link"));
    }

    #[test]
    fn test_validate_symlink_and_resume_conflict() {
        let args = CLIArgs {
            sources: vec![PathBuf::from("source.txt")],
            destination: PathBuf::from("dest.txt"),
            target_directory: None,
            recursive: false,
            parallel: 4,
            resume: true,
            force: false,
            interactive: false,
            parents: false,
            preserve: None,
            attributes_only: false,
            remove_destination: false,
            symbolic_link: Some(SymlinkMode::Auto),
            hard_link: false,
            dereference: true,
            no_dereference: false,
            dereference_command_line: false,
            backup: None,
            reflink: None,
            exclude: Vec::new(),
            no_config: false,
            config: None,
            command: None,
        };

        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("continue"));
    }

    #[test]
    fn test_validate_hardlink_and_resume_conflict() {
        let args = CLIArgs {
            sources: vec![PathBuf::from("source.txt")],
            destination: PathBuf::from("dest.txt"),
            target_directory: None,
            recursive: false,
            parallel: 4,
            resume: true,
            force: false,
            interactive: false,
            parents: false,
            preserve: None,
            attributes_only: false,
            remove_destination: false,
            symbolic_link: None,
            hard_link: true,
            dereference: true,
            no_dereference: false,
            dereference_command_line: false,
            backup: None,
            reflink: None,
            exclude: Vec::new(),
            no_config: false,
            config: None,
            command: None,
        };

        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("link"));
    }

    #[test]
    fn test_validate_success() {
        let args = CLIArgs {
            sources: vec![PathBuf::from("source.txt")],
            destination: PathBuf::from("dest.txt"),
            target_directory: None,
            recursive: false,
            parallel: 4,
            resume: false,
            force: false,
            interactive: false,
            parents: false,
            preserve: None,
            attributes_only: false,
            remove_destination: false,
            symbolic_link: None,
            hard_link: false,
            dereference: true,
            no_dereference: false,
            dereference_command_line: false,
            backup: None,
            reflink: None,
            exclude: Vec::new(),
            no_config: false,
            config: None,
            command: None,
        };

        let result = args.validate();
        assert!(result.is_ok());
    }
}
