# file-integrity-rs 🦀

File Integrity Monitor in Rust. Creates a SHA-256 baseline of your files and detects changes.

## Usage
```bash
# Create baseline
file-integrity init /etc/nginx /var/www -o baseline.json

# Check for changes
file-integrity check /etc/nginx /var/www -b baseline.json
```

## Features
- SHA-256 file hashing
- Recursive directory walking
- Detects new, removed, and modified files
- JSON baseline format

## License
MIT
