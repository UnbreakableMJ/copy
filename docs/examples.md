# Examples

This guide provides practical examples of using `copy` for common file copying tasks.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Directory Operations](#directory-operations)
- [Exclude Patterns](#exclude-patterns)
- [Preserve Attributes](#preserve-attributes)
- [Backup Strategies](#backup-strategies)
- [Symlink Operations](#symlink-operations)
- [Hard Links](#hard-links)
- [Resume Interrupted Transfers](#resume-interrupted-transfers)
- [Advanced Scenarios](#advanced-scenarios)
- [Performance Optimization](#performance-optimization)

## Basic Usage

### Copy a Single File
```bash
# Simple copy
copy source.txt destination.txt

# Copy to directory (destination filename matches source)
copy source.txt /path/to/directory/

# Copy with overwrite confirmation
copy -i source.txt destination.txt
```

### Copy Multiple Files
```bash
# Copy multiple files to a directory
copy file1.txt file2.txt file3.txt /destination/

# Using target directory flag
copy -t /destination/ file1.txt file2.txt file3.txt

# Copy with pattern expansion
copy *.txt /destination/
```

### Force Overwrite
```bash
# Overwrite read-only files
copy -f source.txt destination.txt

# Remove destination before copying
copy --remove-destination source.txt destination.txt
```

## Directory Operations

### Copy Directory Recursively
```bash
# Basic recursive copy
copy -r source_dir/ destination_dir/

# Copy with specific parallel
copy -r -j 8 source_dir/ destination_dir/

# Interactive recursive copy
copy -ri source_dir/ destination_dir/
```

### Preserve Directory Structure
```bash
# Copy with parent directories
copy --parents src/components/Button.tsx /backup/

# Result: /backup/src/components/Button.tsx

# Multiple files with parents
copy --parents src/**/*.tsx /backup/
```

### Copy Only Directory Structure (No Files)
```bash
# Copy attributes only (creates dirs, updates permissions)
copy -r --attributes-only source_dir/ destination_dir/
```

## Exclude Patterns

### Exclude by File Extension
```bash
# Exclude temporary files
copy -r -e "*.tmp" -e "*.swp" source/ dest/

# Exclude multiple patterns at once
copy -r -e "*.tmp,*.log,*.cache" source/ dest/
```

### Exclude Directories
```bash
# Exclude node_modules
copy -r -e "node_modules" project/ backup/

# Exclude multiple directories
copy -r -e "node_modules" -e ".git" -e "target" project/ backup/

# Exclude with comma-separated list
copy -r -e "node_modules,.git,__pycache__" project/ backup/
```

### Complex Exclusion Patterns
```bash
# Exclude build artifacts from multiple languages
copy -r \
  -e "node_modules" \
  -e "target" \
  -e "dist" \
  -e "__pycache__" \
  -e "*.pyc" \
  -e ".git" \
  project/ backup/

# Exclude with glob patterns
copy -r -e "test_*.py" -e "*.test.js" source/ dest/

# Exclude specific paths
copy -r -e "src/generated/*" -e "docs/api/*" project/ backup/
```

### Development Project Backup
```bash
# Skip all common development files
copy -r \
  -e "node_modules" \
  -e ".git" \
  -e ".svn" \
  -e "target" \
  -e "build" \
  -e "dist" \
  -e "*.pyc" \
  -e "__pycache__" \
  -e ".venv" \
  -e ".env" \
  -e ".DS_Store" \
  -e "Thumbs.db" \
  my-project/ /backup/my-project/
```

## Preserve Attributes

### Preserve Default Attributes
```bash
# Preserve mode, ownership, and timestamps (default with -p)
copy -p source.txt destination.txt

# Same as above (empty -p uses defaults)
copy -p= source.txt destination.txt
```

### Preserve Specific Attributes
```bash
# Preserve only file permissions
copy -p=mode source.txt destination.txt

# Preserve permissions and timestamps
copy -p=mode,timestamps source.txt destination.txt

# Preserve everything
copy -p=all source.txt destination.txt
# Or use --attributes-only for dirs
copy -r --attributes-only source/ dest/
```

### Preserve Hard Link Relationships
```bash
# Preserve hard links between files
copy -r -p=links source/ dest/

# This maintains hard link relationships in the destination
# If source/file1 and source/file2 are hard linked,
# dest/file1 and dest/file2 will also be hard linked
```

### Update Only Attributes
```bash
# Update permissions without copying data
copy -p=mode --attributes-only source.txt destination.txt

# Update all attributes for directory tree
copy -r -p=all --attributes-only source/ dest/
```

## Backup Strategies

### Simple Backup
```bash
# Append ~ to existing files
copy -b source.txt destination.txt

# If destination.txt exists, it becomes destination.txt~
```

### Numbered Backups
```bash
# Create numbered backups
copy -b=numbered source.txt destination.txt

# Creates:
# - destination.txt (new file)
# - destination.txt.~1~ (first backup)
# - destination.txt.~2~ (second backup, if run again)
```

### Smart Backup Strategy
```bash
# Use existing: numbered if backups exist, simple otherwise
copy -b=existing source.txt destination.txt
```

### Backup Entire Directory
```bash
# Create numbered backups for all files
copy -r -b=numbered source/ dest/

# Useful for incremental updates
copy -r -b=numbered --resume updated_project/ production/
```

### Production Deployment with Backup
```bash
# Safe deployment with interactive mode and backups
copy -ri -b=numbered \
  -e "*.log" \
  -e "tmp/*" \
  new_version/ /var/www/production/
```

## Symlink Operations

### Create Symbolic Links
```bash
# Create symlinks instead of copying (auto mode)
copy -s source.txt link.txt
copy -s source_dir/ link_dir/

# Create absolute symlinks
copy -s=absolute source.txt /path/to/link.txt

# Create relative symlinks
copy -s=relative source.txt ../links/link.txt
```

### Copy Symlink Behavior
```bash
# Don't follow symlinks (copy the link itself)
copy -P -r source/ dest/

# Follow all symlinks (copy target files)
copy -L -r source/ dest/

# Follow only command-line symlinks
copy -H link_to_dir/ dest/
```

### Create Link Farm
```bash
# Create relative symlinks to all files
copy -r -s=relative /media/music/ ~/music-links/

# Result: ~/music-links/ contains symlinks to originals
```

### Mirror with Symlinks
```bash
# Create symlink mirror of directory structure
copy -r -s=relative \
  -e ".git" \
  ~/projects/my-app/ ~/links/my-app/
```

## Hard Links

### Create Hard Links
```bash
# Hard link instead of copying
copy -l source.txt hardlink.txt

# Hard link multiple files
copy -l file1.txt file2.txt file3.txt /destination/

# Hard link directory contents
copy -rl source/ dest/
```

### Space-Efficient Backups
```bash
# Create hard-linked backup (saves space)
copy -rl \
  -e "*.log" \
  -e "tmp/*" \
  /var/www/app/ /backup/snapshots/2025-01-24/

# Files unchanged from source share inodes
# Modified files get new inodes
```

### Deduplication
```bash
# Create hard links to deduplicate identical files
copy -rl -p=links source/ deduplicated/
```

## Resume Interrupted Transfers

### Resume Large Copy Operation
```bash
# Start copy
copy -r large_dataset/ /backup/large_dataset/

# If interrupted (Ctrl+C), resume with:
copy -r --resume large_dataset/ /backup/large_dataset/

# Files already copied are skipped (verified by checksum)
```

### Resume with Progress
```bash
# Resume large transfer with detailed progress
copy -r --resume -j 8 \
  /mnt/source/big_project/ \
  /mnt/backup/big_project/

# Shows: "Skipping X files that already exist"
```

### Smart Resume
```bash
# Resume only copies files that are:
# - Missing in destination
# - Different size
# - Different content (checksum verified)
# - Older modification time in source

copy -r --resume source/ dest/
```

## Advanced Scenarios

### Copy-on-Write (Reflink) Copies
```bash
# Instant copy on supporting filesystems (Btrfs, XFS, APFS)
copy --reflink source.txt destination.txt

# Require reflink (fail if not supported)
copy --reflink=always source.txt destination.txt

# Try reflink, fall back to regular copy
copy --reflink=auto source.txt destination.txt
```

### Fast Snapshot on Btrfs
```bash
# Instant snapshot using reflinks
copy -r --reflink=always /home/user/ /snapshots/user-2025-01-24/

# Takes seconds regardless of size
# Space only used when files are modified
```

### Sync-like Behavior
```bash
# Update destination with newer files
copy -r --resume \
  -p=mode,timestamps \
  source/ dest/
```

### Migrate with Verification
```bash
# Copy with full preservation and resume capability
copy -r --resume \
  -p=all \
  -b=numbered \
  -j 16 \
  /old/server/data/ /new/server/data/
```

### Clone Git Repository (Files Only)
```bash
# Copy git repo without .git directory
copy -r -e ".git" my-project/ my-project-copy/
```

### Selective Directory Sync
```bash
# Sync only specific file types
copy -r \
  -e "!*.txt" \
  -e "!*.md" \
  -e "*" \
  source/ dest/

# Note: Exclude all (*), then include specific types
```

### Archive with Structure
```bash
# Archive with full paths preserved
copy --parents \
  src/**/*.{rs,toml} \
  tests/**/*.rs \
  /archive/
```

### Update Permissions Recursively
```bash
# Update only permissions, don't copy data
copy -r --attributes-only -p=mode template/ project/
```

## Performance Optimization

### Maximize Throughput
```bash
# Use high parallel for many small files
copy -r -j 16 many_small_files/ dest/

# Use lower parallel for large files
copy -r -j 2 few_large_files/ dest/
```

### Fast Local Copy (SSD to SSD)
```bash
# Maximum speed with reflink
copy -r --reflink=auto -j 8 source/ dest/
```

### Network Copy Optimization
```bash
# Lower parallel, resume support
copy -r -j 4 --resume /local/data/ /network/mount/data/
```

### Large Dataset Transfer
```bash
# Optimized for large transfers
copy -r \
  -j 8 \
  --resume \
  -p=mode,timestamps \
  /source/terabytes/ /dest/terabytes/

# If interrupted:
# - Resume with same command
# - Already-copied files are skipped
# - Partial files are re-copied
```

### Minimal Overhead Copy
```bash
# Skip all preservation for maximum speed
copy -r -j 16 source/ dest/

# No progress bar, no preservation
# Fastest possible copy
```

## Real-World Workflows

### Daily Development Backup
```bash
# Backup work directory, excluding build artifacts
copy -r --resume \
  -b=numbered \
  -e "node_modules,.git,target,dist,build" \
  ~/projects/my-app/ \
  /backup/daily/my-app-$(date +%Y-%m-%d)/
```

### Deploy Web Application
```bash
# Deploy with backup and verification
copy -ri \
  -b=numbered \
  -p=mode,timestamps \
  -e "*.log" \
  -e "uploads/*" \
  -e ".env" \
  ./dist/ /var/www/production/
```

### Create Development Environment
```bash
# Clone project template without version control
copy -r \
  -e ".git" \
  -e "node_modules" \
  -e ".env" \
  ~/templates/react-app/ \
  ~/projects/new-project/
```

### Photo Library Backup
```bash
# Backup photos with preservation
copy -r \
  -p=all \
  -b=numbered \
  --resume \
  -j 8 \
  ~/Pictures/ \
  /media/backup/Pictures-$(date +%Y-%m-%d)/
```

### Server Migration
```bash
# Migrate server data with full preservation
copy -r \
  --resume \
  -p=all \
  -e "*.log" \
  -e "tmp/*" \
  -j 4 \
  /old/server/data/ /new/server/data/
```

### Create Project Snapshot
```bash
# Fast snapshot using reflinks (Btrfs/XFS/APFS)
copy -r --reflink=always \
  ~/projects/my-app/ \
  ~/snapshots/my-app-$(date +%Y-%m-%d-%H%M)/
```

### Deduplicated Backup
```bash
# Create hard-linked backup (saves space)
copy -rl \
  -p=all \
  ~/Documents/ \
  /backup/incremental/$(date +%Y-%m-%d)/

# Unchanged files share inodes with previous backups
```

### Cross-Platform Copy
```bash
# Copy preserving only timestamps (safe for Windows/Linux)
copy -r -p=timestamps source/ dest/
```

## Combining Options

### Safe Production Update
```bash
copy -ri \
  --resume \
  -b=numbered \
  -p=mode,timestamps \
  -e "*.log" \
  -e "tmp" \
  -e ".env" \
  ./build/ /var/www/production/
```

**Explanation:**
- `-r`: Recursive copy
- `-i`: Interactive (confirm overwrites)
- `--resume`: Skip already-copied files
- `-b=numbered`: Create numbered backups
- `-p=mode,timestamps`: Preserve permissions and timestamps
- `-e`: Exclude logs, temp files, and environment files

### Fast Bulk Transfer
```bash
copy -r \
  -j 16 \
  --resume \
  --reflink=auto \
  -e ".git" \
  -e "node_modules" \
  ~/projects/ /backup/projects/
```

**Explanation:**
- `-j 16`: High parallel
- `--resume`: Resume if interrupted
- `--reflink=auto`: Use CoW if available
- `-e`: Exclude large directories

### Complete Preservation
```bash
copy -r \
  -p=all \
  --resume \
  -b=numbered \
  source/ dest/
```

**Explanation:**
- `-p=all`: Preserve everything (mode, ownership, timestamps, xattr, context, links)
- `--resume`: Resume capability
- `-b=numbered`: Backup existing files

## Tips and Tricks

### Dry Run Simulation

While `copy` doesn't have a built-in dry-run, you can test with:
```bash
# Use attributes-only to test without copying data
copy -r --attributes-only source/ dest/
```

### Check What Will Be Excluded
```bash
# Use a test directory to verify exclude patterns
copy -r -e "*.tmp" -e "node_modules" test_source/ test_dest/
```

### Resume After System Crash
```bash
# Always safe to resume
copy -r --resume /backup/incomplete/ /restore/location/

# Checksums verify file integrity
```

### Space-Efficient Testing
```bash
# Use symlinks for testing
copy -r -s=relative test_data/ test_copy/

# Use hard links to save space
copy -rl test_data/ test_copy/
```

---

For configuration options, see [Configuration Guide](configuration.md).

For CLI reference, see `copy --help`.
