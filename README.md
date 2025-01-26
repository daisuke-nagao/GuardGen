# GuardGen

## Overview

GuardGen is a Rust-based command-line tool designed to generate unique header guards for C/C++ header files. It uses UUID version 7 to ensure the guards are unique and avoids potential naming conflicts.

## Features

- Generates a unique UUID-based header guard in the following format:
  ```c
  #ifndef UUID_<UUID>
  #define UUID_<UUID>

  
  #endif /* UUID_<UUID> */
  ```
  where `<UUID>` is a UUIDv7 string with dashes replaced by underscores and converted to uppercase.

- Outputs the header guard to a file or the standard output.
- Supports optional overwriting of existing files.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/) installed on your system.
2. Clone the repository:
   ```bash
   git clone https://github.com/daisuke-nagao/GuardGen.git
   cd guardgen
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The compiled binary will be available in the `target/release` directory.

## Usage

```bash
./guardgen [--output <file>] [--overwrite]
```

### Options

- `--output <file>` or `-o <file>`: Specifies the file to save the generated header guard. If not provided, the guard will be printed to the standard output.
- `--overwrite`: Allows overwriting an existing file. If not specified and the file already exists, the program will exit with an error.

### Examples

1. Generate a header guard and print it to the terminal:
   ```bash
   ./guardgen
   ```

2. Save the header guard to a file:
   ```bash
   ./guardgen --output my_header.h
   ```

3. Overwrite an existing file:
   ```bash
   ./guardgen --output my_header.h --overwrite
   ```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Author

Daisuke Nagao

---

GuardGen is designed to simplify the creation of header guards, ensuring consistency and avoiding potential conflicts in C/C++ projects.

