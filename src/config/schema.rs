use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ExcludeConfig {
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CopyConfig {
    pub parallel: usize,
    pub recursive: bool,
    pub parents: bool,
    pub force: bool,
    pub interactive: bool,
    pub resume: bool,
    pub attributes_only: bool,
    pub remove_destination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreserveConfig {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SymlinkConfig {
    pub mode: String,   // "auto", "absolute", "relative"
    pub follow: String, // "never", "always", "command-line"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupConfig {
    pub mode: String, // "none", "simple", "numbered", "existing"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReflinkConfig {
    pub mode: String, // "auto", "always", "never"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgressConfig {
    pub style: String, // "default", "detailed"
    pub bar: ProgressBarConfig,
    pub color: ProgressColorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgressBarConfig {
    pub filled: String,
    pub empty: String,
    pub head: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProgressColorConfig {
    pub bar: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub exclude: ExcludeConfig,
    pub copy: CopyConfig,
    pub preserve: PreserveConfig,
    pub symlink: SymlinkConfig,
    pub backup: BackupConfig,
    pub reflink: ReflinkConfig,
    pub progress: ProgressConfig,
}

impl Default for CopyConfig {
    fn default() -> Self {
        Self {
            parallel: 4,
            recursive: false,
            parents: false,
            force: false,
            interactive: false,
            resume: false,
            attributes_only: false,
            remove_destination: false,
        }
    }
}

impl Default for PreserveConfig {
    fn default() -> Self {
        Self {
            mode: "default".to_string(),
        }
    }
}

impl Default for SymlinkConfig {
    fn default() -> Self {
        Self {
            mode: "".to_string(),
            follow: "".to_string(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            mode: "none".to_string(),
        }
    }
}

impl Default for ReflinkConfig {
    fn default() -> Self {
        Self {
            mode: "".to_string(),
        }
    }
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            style: "default".to_string(),
            bar: ProgressBarConfig::default(),
            color: ProgressColorConfig::default(),
        }
    }
}

impl Default for ProgressBarConfig {
    fn default() -> Self {
        Self {
            filled: "█".to_string(),
            empty: "░".to_string(),
            head: "░".to_string(),
        }
    }
}

impl Default for ProgressColorConfig {
    fn default() -> Self {
        Self {
            bar: "white".to_string(),
            message: "white".to_string(),
        }
    }
}

impl Config {
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn merge_partial(&mut self, partial: PartialConfig) {
        if let Some(exclude) = partial.exclude
            && let Some(patterns) = exclude.patterns
        {
            self.exclude.patterns.extend(patterns);
        }

        if let Some(copy) = partial.copy {
            if let Some(parallel) = copy.parallel {
                self.copy.parallel = parallel;
            }
            if let Some(recursive) = copy.recursive {
                self.copy.recursive = recursive;
            }
            if let Some(parents) = copy.parents {
                self.copy.parents = parents;
            }
            if let Some(force) = copy.force {
                self.copy.force = force;
            }
            if let Some(interactive) = copy.interactive {
                self.copy.interactive = interactive;
            }
            if let Some(resume) = copy.resume {
                self.copy.resume = resume;
            }
            if let Some(attributes_only) = copy.attributes_only {
                self.copy.attributes_only = attributes_only;
            }
            if let Some(remove_destination) = copy.remove_destination {
                self.copy.remove_destination = remove_destination;
            }
        }

        if let Some(preserve) = partial.preserve
            && let Some(mode) = preserve.mode
        {
            self.preserve.mode = mode;
        }

        if let Some(symlink) = partial.symlink {
            if let Some(mode) = symlink.mode {
                self.symlink.mode = mode;
            }
            if let Some(follow) = symlink.follow {
                self.symlink.follow = follow;
            }
        }

        if let Some(backup) = partial.backup
            && let Some(mode) = backup.mode
        {
            self.backup.mode = mode;
        }

        if let Some(reflink) = partial.reflink
            && let Some(mode) = reflink.mode
        {
            self.reflink.mode = mode;
        }

        if let Some(progress) = partial.progress {
            if let Some(style) = progress.style {
                self.progress.style = style;
            }
            if let Some(bar) = progress.bar {
                if let Some(filled) = bar.filled {
                    self.progress.bar.filled = filled;
                }
                if let Some(empty) = bar.empty {
                    self.progress.bar.empty = empty;
                }
                if let Some(head) = bar.head {
                    self.progress.bar.head = head;
                }
            }
            if let Some(color) = progress.color {
                if let Some(bar) = color.bar {
                    self.progress.color.bar = bar;
                }
                if let Some(message) = color.message {
                    self.progress.color.message = message;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialExcludeConfig {
    pub patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialCopyConfig {
    pub parallel: Option<usize>,
    pub recursive: Option<bool>,
    pub parents: Option<bool>,
    pub force: Option<bool>,
    pub interactive: Option<bool>,
    pub resume: Option<bool>,
    pub attributes_only: Option<bool>,
    pub remove_destination: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialPreserveConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialSymlinkConfig {
    pub mode: Option<String>,
    pub follow: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialBackupConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialReflinkConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialProgressConfig {
    pub style: Option<String>,
    pub bar: Option<PartialProgressBarConfig>,
    pub color: Option<PartialProgressColorConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialProgressBarConfig {
    pub filled: Option<String>,
    pub empty: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialProgressColorConfig {
    pub bar: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartialConfig {
    pub exclude: Option<PartialExcludeConfig>,
    pub copy: Option<PartialCopyConfig>,
    pub preserve: Option<PartialPreserveConfig>,
    pub symlink: Option<PartialSymlinkConfig>,
    pub backup: Option<PartialBackupConfig>,
    pub reflink: Option<PartialReflinkConfig>,
    pub progress: Option<PartialProgressConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_partial_preserves_omitted_defaults() {
        let mut config = Config::default();
        config.merge_partial(PartialConfig {
            copy: Some(PartialCopyConfig {
                recursive: Some(true),
                ..PartialCopyConfig::default()
            }),
            ..PartialConfig::default()
        });

        assert!(config.copy.recursive);
        assert_eq!(config.copy.parallel, 4);
        assert_eq!(config.backup.mode, "none");
    }

    #[test]
    fn merge_partial_appends_exclude_patterns() {
        let mut config = Config::default();
        config.exclude.patterns.push("system.tmp".to_string());

        config.merge_partial(PartialConfig {
            exclude: Some(PartialExcludeConfig {
                patterns: Some(vec!["project.tmp".to_string()]),
            }),
            ..PartialConfig::default()
        });

        assert_eq!(config.exclude.patterns, ["system.tmp", "project.tmp"]);
    }
}
