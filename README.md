# JSON Diff Checker

A powerful JSON file difference checking tool written in Rust, supporting recursive comparison of JSON file structures and values.

## 🚀 Features

- **Recursive Comparison**: Deep traversal of JSON structures, checking all nested paths
- **Flexible Comparison Modes**: 
  - Structure-only comparison (default)
  - Value comparison mode
  - Type-only comparison mode
- **Multi-file Support**: Compare multiple files against a base file at once
- **Colored Output**: Clear colored terminal output for quick difference identification
- **Export Functionality**: Export comparison results to JSON format
- **Detailed Statistics**: Comprehensive comparison statistics
- **Path Handling**: Intelligent handling of JSON keys with special characters

## 📦 Installation

### Build from Source

Make sure you have [Rust](https://rustup.rs/) installed, then run:

```bash
git clone https://github.com/mgher668/json-diff-checker.git
cd json-diff-checker
cargo build --release
```

The compiled executable will be located at `target/release/json_diff_checker`.

### Direct Run

```bash
cargo run -- [OPTIONS] <BASE_FILE> <COMPARE_FILES>...
```

## 🔧 Usage

### Basic Usage

```bash
# Compare structure of two JSON files
json_diff_checker base.json compare.json

# Compare multiple files
json_diff_checker base.json file1.json file2.json file3.json
```

### Advanced Options

```bash
# Enable value comparison (check not only structure but also values)
json_diff_checker -v base.json compare.json

# Compare types only, ignore specific value differences
json_diff_checker -v -t base.json compare.json

# Show brief summary
json_diff_checker -s base.json compare.json

# Export results to JSON file
json_diff_checker -e results.json base.json compare.json

# Include parent path information in missing items
json_diff_checker -p base.json compare.json
```

### Command Line Arguments

| Argument | Short | Description |
|----------|-------|-------------|
| `--check-values` | `-v` | Check values as well as structure |
| `--type-only` | `-t` | Only check types, ignore value differences (requires `-v`) |
| `--summary` | `-s` | Show only summary |
| `--export` | `-e` | Export results to JSON file |
| `--include-parents` | `-p` | Include parent paths in missing items |

## 📋 Output Examples

### Detailed Output Mode

```
═══════════════════════════════════════════════════════════════════════════════
JSON Diff Checker
═══════════════════════════════════════════════════════════════════════════════
Base file: base.json
Total items: 15
Value checking: Full comparison
────────────────────────────────────────────────────────────────────────────────

▶ compare.json

  ✗ Missing paths (2):
    └ user.profile.avatar
    └ settings.theme

  ≠ Different values (1):
    └ user.age
      expected: 25
      actual: 30

  ✓ All other items match!
```

### Summary Mode

```
compare1.json              ✓ OK
compare2.json              2 missing, 1 different
compare3.json              1 type mismatch
```

## 🏗️ Project Structure

```
json-diff-checker/
├── src/
│   ├── main.rs          # Command line interface and main program logic
│   └── lib.rs           # Core JSON comparison functionality
├── tests/
│   └── test_data/       # Test JSON files
├── Cargo.toml           # Project configuration and dependencies
└── README.md            # Project documentation
```

## 🔍 Core Features

### JSON Path Parsing

The tool can intelligently handle complex JSON paths, including:
- Nested objects: `user.profile.name`
- Array indices: `items[0].id`
- Special character keys: `["key.with.dots"]`

### Comparison Modes

1. **Structure Comparison** (default): Only check if JSON structure matches
2. **Value Comparison**: Check both structure and values for exact match
3. **Type Comparison**: Only check data types, ignore specific values

### Output Formats

- **Colored Terminal Output**: Use different colors to identify different types of differences
- **JSON Export**: Structured comparison results for programmatic processing
- **Statistics**: Detailed comparison statistics

## 🧪 Testing

Run test suite:

```bash
cargo test
```

Run performance benchmarks:

```bash
cargo bench
```

## 📄 Dependencies

- `serde` - JSON serialization/deserialization
- `serde_json` - JSON processing
- `clap` - Command line argument parsing
- `anyhow` - Error handling
- `colored` - Colored terminal output

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Related Links

- [Rust Official Website](https://www.rust-lang.org/)
- [Serde JSON Documentation](https://docs.serde.rs/serde_json/)
- [Clap Documentation](https://docs.rs/clap/) 