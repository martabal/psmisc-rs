# fuser

Rust implementation of `fuser`.

Identifies processes using specified files or directories.

## Features

- Find processes using files or directories
- Check current working directory, root directory, executable, file descriptors, and memory maps
- Kill processes using specified files
- Custom signal support
- Verbose output showing access types
- Quiet mode for scripting

## Usage

```bash
# Find processes using a file
fuser /path/to/file

# Verbose output
fuser -v /path/to/file

# Kill processes using a file
fuser -k /path/to/file

# Kill with custom signal
fuser -k -s TERM /path/to/file
```
