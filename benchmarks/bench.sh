#!/usr/bin/env bash
# Copy vs GNU cp — Real-World Benchmark (Cold + Warm Cache)
set -euo pipefail

# ----------------------------------------------------------------------------
# COLORS
# ----------------------------------------------------------------------------
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# ----------------------------------------------------------------------------
# CONFIG
# ----------------------------------------------------------------------------
BENCH_DIR="${BENCH_DIR:-/tmp/copy_multi_bench}"
REPOS_DIR="$BENCH_DIR/repos"
THREADS="${THREADS:-$(nproc)}"
RUNS="${RUNS:-6}"

MODE="${1:-warm}"  # warm | cold

# ----------------------------------------------------------------------------
# FIND Copy BINARY
# ----------------------------------------------------------------------------
find_copy() {
    # 1. Honour explicit COPY_PATH environment variable
    if [ -n "${COPY_PATH:-}" ] && [ -x "$COPY_PATH" ]; then
        echo "$COPY_PATH"
        return 0
    fi

    # 2. Check if copy is already on PATH
    if command -v copy &>/dev/null; then
        command -v copy
        return 0
    fi

    # 3. Common install locations
    local candidates=(
        "$HOME/.local/bin/copy"
        "$HOME/.cargo/bin/copy"
        "/usr/local/bin/copy"
        "/usr/bin/copy"
    )

    for candidate in "${candidates[@]}"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    # 4. Look relative to the script / repo directory
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local repo_dir
    repo_dir="$(dirname "$script_dir")"

    for candidate in "$repo_dir/copy" "$repo_dir/target/release/copy" "$repo_dir/target/debug/copy"; do
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return 0
        fi
    done

    return 1
}

# ----------------------------------------------------------------------------
# PRECHECKS
# ----------------------------------------------------------------------------
echo -e "${GREEN}=== Copy vs GNU cp Benchmark ($MODE cache) ===${NC}"
echo ""

COPY_PATH="$(find_copy || true)"

if [ -z "$COPY_PATH" ]; then
    echo -e "${RED}Error: copy binary not found.${NC}"
    echo ""
    echo -e "${YELLOW}Searched in:${NC}"
    echo "  • \$COPY_PATH environment variable"
    echo "  • \$PATH (command -v copy)"
    echo "  • ~/.local/bin/copy"
    echo "  • ~/.cargo/bin/copy"
    echo "  • /usr/local/bin/copy"
    echo "  • /usr/bin/copy"
    echo "  • <repo>/copy, <repo>/target/release/copy, <repo>/target/debug/copy"
    echo ""
    echo -e "${YELLOW}Install copy using one of:${NC}"
    echo "  curl -fsSL https://raw.githubusercontent.com/UnbreakableMJ/copy/main/install.sh | bash"
    echo "  cargo install copy"
    echo ""
    read -p "Would you like to install copy now? (Y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        echo -e "${BLUE}Installing copy...${NC}"
        curl -fsSL https://raw.githubusercontent.com/UnbreakableMJ/copy/main/install.sh | bash
        echo ""
        COPY_PATH="$(find_copy || true)"
        if [ -z "$COPY_PATH" ]; then
            echo -e "${RED}Installation succeeded but copy still not found in expected locations.${NC}"
            echo -e "${YELLOW}Try adding ~/.local/bin to your PATH and re-running.${NC}"
            exit 1
        fi
    else
        exit 1
    fi
fi

echo -e "${GREEN}Found copy at: $COPY_PATH${NC}"

if ! command -v hyperfine &>/dev/null; then
    echo -e "${RED}Error: hyperfine not found (cargo install hyperfine)${NC}"
    exit 1
fi

if [ "$MODE" = "cold" ] && [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Cold-cache mode requires sudo/root${NC}"
    exit 1
fi

echo "Benchmark dir : $BENCH_DIR"
echo "CPU cores    : $(nproc)"
echo "Cache mode   : $MODE"
echo ""

# ----------------------------------------------------------------------------
# CACHE CONTROL
# ----------------------------------------------------------------------------


# ----------------------------------------------------------------------------
# CLEAN START
# ----------------------------------------------------------------------------


# ----------------------------------------------------------------------------
# REPOSITORIES (HEAVY REAL-WORLD SET)
# ----------------------------------------------------------------------------
declare -A REPOS=(
    ["linux"]="https://github.com/torvalds/linux.git"
    ["tensorflow"]="https://github.com/tensorflow/tensorflow.git"
    ["rust"]="https://github.com/rust-lang/rust.git"
    ["kubernetes"]="https://github.com/kubernetes/kubernetes.git"
    ["vscode"]="https://github.com/microsoft/vscode.git"
    ["node"]="https://github.com/nodejs/node.git"
    ["go"]="https://github.com/golang/go.git"
    ["chromium"]="https://github.com/chromium/chromium.git"
    ["OpenImageIO"]="https://github.com/AcademySoftwareFoundation/OpenImageIO"
    ["openexr"]="https://github.com/AcademySoftwareFoundation/openexr-images"
    ["godot"]="https://github.com/godotengine/godot-demo-projects"
)

# ----------------------------------------------------------------------------
# CLONE
# ----------------------------------------------------------------------------
echo -e "${YELLOW}Cloning repositories...${NC}"
mkdir -p "$REPOS_DIR"
cd "$REPOS_DIR"

for name in "${!REPOS[@]}"; do
    echo -e "${BLUE}Cloning $name${NC}"
    git clone --depth 1 "${REPOS[$name]}" "$name" >/dev/null
    size=$(du -sh "$name" | cut -f1)
    files=$(find "$name" -type f | wc -l)
    echo -e "${GREEN}✓ $name: $size ($files files)${NC}"
    echo ""
done

total_size=$(du -sh "$REPOS_DIR" | cut -f1)
total_files=$(find "$REPOS_DIR" -type f | wc -l)
total_dirs=$(find "$REPOS_DIR" -type d | wc -l)

echo -e "${YELLOW}Dataset:${NC} $total_size | $total_files files | $total_dirs dirs"
echo ""

# ----------------------------------------------------------------------------
# PER-REPO BENCHMARKS
# ----------------------------------------------------------------------------
echo -e "${YELLOW}Running per-repository benchmarks...${NC}"
echo ""

for name in "${!REPOS[@]}"; do
    src="$REPOS_DIR/$name"
    [ ! -d "$src" ] && continue

    echo -e "${BLUE}━━━━━━━━━━ $name ━━━━━━━━━━${NC}"

    hyperfine \
        --runs "$RUNS" \
        --warmup 0 \
        --prepare "rm -rf $BENCH_DIR/dest_cp $BENCH_DIR/dest_copy; sync; [ \"$MODE\" = cold ] && echo 3 > /proc/sys/vm/drop_caches || true" \
        --export-markdown "$BENCH_DIR/${name}_${MODE}.md" \
        --export-json "$BENCH_DIR/${name}_${MODE}.json" \
        "$COPY_PATH -r -j=$THREADS $src $BENCH_DIR/dest_copy" \
        "cp -r $src $BENCH_DIR/dest_cp"

    echo ""
done

# ----------------------------------------------------------------------------
# FULL DATASET BENCHMARK
# ----------------------------------------------------------------------------
echo -e "${YELLOW}Running FULL dataset benchmark...${NC}"
echo ""

hyperfine \
    --runs "$RUNS" \
    --warmup 0 \
    --prepare "rm -rf $BENCH_DIR/dest_cp $BENCH_DIR/dest_copy; sync; [ \"$MODE\" = cold ] && echo 3 > /proc/sys/vm/drop_caches || true" \
    --export-markdown "$BENCH_DIR/full_${MODE}.md" \
    --export-json "$BENCH_DIR/full_${MODE}.json" \
    "$COPY_PATH -r -j=$THREADS $REPOS_DIR $BENCH_DIR/dest_copy" \
    "cp -r $REPOS_DIR $BENCH_DIR/dest_cp"

# ----------------------------------------------------------------------------
# SUMMARY
# ----------------------------------------------------------------------------
cat > "$BENCH_DIR/SUMMARY_${MODE}.md" <<EOF
# Copy vs GNU cp — $MODE cache benchmark

## Environment
- CPU cores: $(nproc)
- OS: $(uname -s) $(uname -r)
- Date: $(date)
- cp: $(cp --version | head -n1)
- Cache mode: $MODE

## Dataset
- Size: $total_size
- Files: $total_files
- Directories: $total_dirs
- Repositories: ${#REPOS[@]}

EOF

for name in "${!REPOS[@]}"; do
    echo "## $name" >> "$BENCH_DIR/SUMMARY_${MODE}.md"
    cat "$BENCH_DIR/${name}_${MODE}.md" >> "$BENCH_DIR/SUMMARY_${MODE}.md"
    echo "" >> "$BENCH_DIR/SUMMARY_${MODE}.md"
done

echo "## Full Dataset" >> "$BENCH_DIR/SUMMARY_${MODE}.md"
cat "$BENCH_DIR/full_${MODE}.md" >> "$BENCH_DIR/SUMMARY_${MODE}.md"

# ----------------------------------------------------------------------------
# DONE
# ----------------------------------------------------------------------------
echo ""
echo -e "${GREEN}Benchmark complete!${NC}"
echo -e "${GREEN}Results: $BENCH_DIR${NC}"
echo -e "${GREEN}Summary: $BENCH_DIR/SUMMARY_${MODE}.md${NC}"
echo ""

read -p "Delete benchmark data (~$total_size × 3 copies)? (Y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    rm -rf "$BENCH_DIR"
    echo -e "${GREEN}Cleaned up${NC}"
else
    echo -e "${YELLOW}Preserved at $BENCH_DIR${NC}"
fi
