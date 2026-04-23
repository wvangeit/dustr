# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Dustr, please report it responsibly.

**Please do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please send an email to the maintainer directly. You can find contact information on the [GitHub profile](https://github.com/wvangeit).

### What to include

- A description of the vulnerability
- Steps to reproduce the issue
- Any potential impact
- Suggested fix (if available)

### Response timeline

- **Acknowledgement**: We will acknowledge receipt of your report within 7 days.
- **Assessment**: We will assess the vulnerability and determine its impact within 14 days.
- **Fix**: We will work on a fix and release a patched version as soon as possible.

## Security Considerations

Dustr is a disk usage analysis tool that reads filesystem metadata. It does not:

- Execute arbitrary code from scanned directories
- Transmit any data over the network
- Require elevated privileges for normal operation

However, users should be aware that scanning directories with symbolic links could potentially expose information about files outside the intended directory scope.
