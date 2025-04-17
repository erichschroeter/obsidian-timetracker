# TimeTracker
- [TimeTracker](#timetracker)
  - [Usage](#usage)
    - [Installation](#installation)
    - [Running the Program](#running-the-program)
  - [Developing](#developing)
    - [Running Tests](#running-tests)
    - [Code Formatting](#code-formatting)
    - [Linting](#linting)
    - [Continuous Integration](#continuous-integration)

TimeTracker is a command-line tool for parsing Markdown journals to extract time-tracking information and export it to a CSV file.

## Usage

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/erichschroeter/timetracker.git
   cd timetracker
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The binary will be available in the `target/release` directory:
   ```bash
   ./target/release/timetracker
   ```

### Running the Program

Run the program with the following options:
- `-d, --dir <DIRECTORY>`: Specify the directory to search (required).
- `-r, --recursive`: Recurse into subdirectories.
- `-v, --verbosity <LEVEL>`: Set log verbosity level (`error`, `warn`, `info`, `debug`, `trace`).
- `-o, --output <FILE>`: Specify the output CSV file (defaults to stdout).

Example:
```bash
./target/release/timetracker -d ./journals -r -v info -o output.csv
```

## Developing

### Running Tests

To run the unit and integration tests, use:
```bash
cargo test
```

This will execute all tests defined in the `src` directory.

### Code Formatting

Ensure your code is formatted correctly before committing:
```bash
cargo fmt
```

### Linting

Check for common issues using:
```bash
cargo clippy
```

### Continuous Integration

This project uses GitHub Actions for Continuous Integration (CI). The CI pipeline ensures that:
1. All tests pass (`cargo test`).
2. Code formatting is correct (`cargo fmt --check`).

The CI workflow is defined in `.github/workflows/ci.yml` and runs automatically on every push and pull request.

Contributions are welcome! Feel free to open issues or submit pull requests.
