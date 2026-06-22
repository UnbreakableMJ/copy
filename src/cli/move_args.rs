use crate::cli::args::{BackupMode, CopyOptions, ReflinkMode};
use crate::config::loader::{load_config, load_config_file};
use crate::error::{CliError, CliResult};
use crate::utility::exclude::{
    ExcludePattern, ExcludeRules, build_exclude_rules, parse_exclude_pattern_list,
};
use crate::utility::helper::parse_backup_mode;
use crate::utility::preserve::PreserveAttr;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// Default number of parallel operations on the cross-device copy fallback.
const DEFAULT_PARALLEL: usize = 4;

/// Command-line surface for the `move` binary (a `mv` replacement).
///
/// `move` renames in place when possible and falls back to a copy + source
/// removal only when the rename cannot be done atomically (cross-device, or
/// when `--exclude` means part of a directory must be left behind).
#[derive(Parser, Debug)]
#[command(name = "move", version = env!("CARGO_PKG_VERSION"))]
pub struct MoveArgs {
    #[arg(help = "Source file(s) or directory(ies)", required = true)]
    pub sources: Vec<PathBuf>,

    #[arg(help = "Destination file or directory", required = true)]
    pub destination: PathBuf,

    #[arg(
        short = 't',
        long = "target-directory",
        value_name = "DIRECTORY",
        help = "move all SOURCE arguments into DIRECTORY"
    )]
    pub target_directory: Option<PathBuf>,

    #[arg(short = 'i', long, help = "prompt before overwrite")]
    pub interactive: bool,

    #[arg(short = 'f', long, help = "do not prompt before overwriting")]
    pub force: bool,

    #[arg(
        short = 'n',
        long = "no-clobber",
        help = "do not overwrite an existing file"
    )]
    pub no_clobber: bool,

    #[arg(
        short = 'u',
        long,
        help = "move only when the SOURCE is newer than the destination, or the destination is missing"
    )]
    pub update: bool,

    #[arg(
        short = 'b',
        long = "backup",
        value_name = "CONTROL",
        default_missing_value = "existing",
        num_args = 0..=1,
        require_equals = true,
        help = "make a backup of each existing destination file (none, numbered, existing, simple)"
    )]
    pub backup: Option<BackupMode>,

    #[arg(short = 'v', long, help = "explain what is being done")]
    pub verbose: bool,

    #[arg(
        short = 'j',
        value_name = "N",
        help = "number of parallel operations on the cross-device copy fallback [default: 4]"
    )]
    pub parallel: Option<usize>,

    #[arg(
        long = "reflink",
        value_name = "WHEN",
        default_missing_value = "auto",
        num_args = 0..=1,
        require_equals = true,
        help = "on the cross-device fallback, control clone/CoW copies (auto, always, never)"
    )]
    pub reflink: Option<ReflinkMode>,

    #[arg(
        short = 'e',
        long = "exclude",
        value_name = "PATTERN",
        help = "exclude files matching pattern on a directory move (can be repeated, comma-separated)"
    )]
    pub exclude: Vec<String>,

    #[arg(
        short = 'p',
        long = "preserve",
        value_name = "ATTR_LIST",
        default_missing_value = "",
        num_args = 0..=1,
        require_equals = true,
        help = "attributes to preserve on the cross-device fallback [default: all]"
    )]
    pub preserve: Option<String>,

    #[arg(long, value_name = "PATH", help = "use custom config file")]
    pub config: Option<PathBuf>,

    #[arg(long, help = "ignore all config files")]
    pub no_config: bool,
}

/// Validated, merged options driving a move operation.
///
/// The `force`/`interactive`/`no_clobber`/`update`/`backup` fields govern the
/// overwrite policy applied before any rename; the remaining fields seed the
/// [`CopyOptions`] used only on the cross-device copy fallback.
#[derive(Debug, Clone)]
pub struct MoveOptions {
    pub force: bool,
    pub interactive: bool,
    pub no_clobber: bool,
    pub update: bool,
    pub backup: Option<BackupMode>,
    pub verbose: bool,
    pub parallel: usize,
    pub reflink: Option<ReflinkMode>,
    pub preserve: PreserveAttr,
    pub exclude_rules: Option<ExcludeRules>,
    pub abort: Arc<AtomicBool>,
}

impl MoveOptions {
    /// Build the [`CopyOptions`] used for the cross-device / exclude copy
    /// fallback. The overwrite policy has already been applied at the move
    /// layer, so the copy runs with `force` and preserves attributes.
    pub fn to_copy_options(&self) -> CopyOptions {
        let mut options = CopyOptions::none();
        options.recursive = true;
        options.force = true;
        options.preserve = self.preserve;
        options.parallel = self.parallel;
        options.reflink = self.reflink;
        options.exclude_rules = self.exclude_rules.clone();
        options.abort = self.abort.clone();
        options
    }
}

impl MoveArgs {
    /// Parse command-line arguments for the `move` binary.
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }

    /// Validate the parsed arguments, merge config defaults, and produce the
    /// `(sources, destination, options)` triple the binary dispatches on.
    pub fn validate(self) -> CliResult<(Vec<PathBuf>, PathBuf, MoveOptions)> {
        // Load the shared `copyconfig.toml` for the general defaults `move`
        // honors (parallel, backup, exclude patterns). `--no-config` skips it;
        // `--config PATH` forces a specific file.
        let config = if self.no_config {
            None
        } else if let Some(ref path) = self.config {
            Some(load_config_file(path).map_err(CliError::Config)?)
        } else {
            Some(load_config().map_err(CliError::Config)?)
        };

        let mut config_parallel = None;
        let mut config_backup = None;
        let mut patterns: Vec<ExcludePattern> = Vec::new();
        if let Some(cfg) = &config {
            config_parallel = Some(cfg.copy.parallel);
            config_backup = parse_backup_mode(&cfg.backup.mode);
            for pattern in &cfg.exclude.patterns {
                patterns.extend(parse_exclude_pattern_list(pattern).map_err(CliError::Exclude)?);
            }
        }

        for pattern in &self.exclude {
            patterns.extend(parse_exclude_pattern_list(pattern).map_err(CliError::Exclude)?);
        }
        let exclude_rules = build_exclude_rules(patterns).map_err(CliError::Exclude)?;

        // Default is to preserve everything on a cross-device move (mv keeps
        // all attributes); `-p` narrows that selection.
        let preserve = match &self.preserve {
            Some(spec) if !spec.is_empty() => {
                PreserveAttr::from_string(spec).map_err(CliError::Preserve)?
            }
            _ => PreserveAttr::all(),
        };

        // CLI `-j` overrides config; otherwise fall back to the config value
        // or the built-in default.
        let parallel = self
            .parallel
            .or(config_parallel)
            .unwrap_or(DEFAULT_PARALLEL);

        let backup = self.backup.or(config_backup);

        // `-t DIR` moves every SOURCE (including the positional "destination")
        // into DIR, mirroring the copy binary.
        let (sources, destination) = if let Some(target) = self.target_directory {
            let mut sources = self.sources;
            sources.push(self.destination);
            (sources, target)
        } else {
            (self.sources, self.destination)
        };

        let options = MoveOptions {
            force: self.force,
            interactive: self.interactive,
            no_clobber: self.no_clobber,
            update: self.update,
            backup,
            verbose: self.verbose,
            parallel,
            reflink: self.reflink,
            preserve,
            exclude_rules,
            abort: Arc::new(AtomicBool::new(false)),
        };

        Ok((sources, destination, options))
    }
}
