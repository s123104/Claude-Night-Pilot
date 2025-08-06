# Security Policy

## Supported Versions

We release patches for security vulnerabilities. The following versions are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

The Claude Night Pilot team and community take security bugs seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report Security Issues

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them by emailing security@claude-night-pilot.dev (if available) or create a private security advisory through GitHub's security tab.

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

### What to Include

Please include the requested information listed below (as much as you can provide) to help us better understand the nature and scope of the possible issue:

* Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
* Full paths of source file(s) related to the manifestation of the issue
* The location of the affected source code (tag/branch/commit or direct URL)
* Any special configuration required to reproduce the issue
* Step-by-step instructions to reproduce the issue
* Proof-of-concept or exploit code (if possible)
* Impact of the issue, including how an attacker might exploit the issue

### Preferred Languages

We prefer all communications to be in English or Traditional Chinese.

## Security Best Practices

### For Users

1. **Keep Claude CLI Updated**: Always use the latest version of Claude CLI
2. **Secure Your Environment**: Keep your operating system and dependencies updated
3. **Review Prompts**: Be cautious when executing prompts from untrusted sources
4. **File Permissions**: Ensure proper file system permissions are set
5. **Network Security**: Use secure networks when accessing Claude APIs

### For Developers

1. **Input Validation**: All user inputs are validated and sanitized
2. **SQL Injection Prevention**: We use parameterized queries exclusively
3. **File System Security**: Access is restricted to application data directory
4. **Command Injection Prevention**: Dangerous command patterns are detected
5. **Dependency Security**: Regular security audits of dependencies

## Security Features

### Built-in Security

- ✅ Input validation and sanitization
- ✅ SQL injection prevention (parameterized queries)
- ✅ File access restrictions
- ✅ Dangerous command detection
- ✅ Execution permission validation
- ✅ Security risk assessment (Low/Medium/High/Critical)
- ✅ Audit logging with SHA256 prompt hashing

### Security Architecture

- **Principle of Least Privilege**: Components have minimal required permissions
- **Defense in Depth**: Multiple layers of security controls
- **Fail-Safe Defaults**: Secure defaults with explicit opt-in for risky operations
- **Input Validation**: All inputs validated at entry points
- **Audit Trail**: Comprehensive logging of security-relevant events

## Vulnerability Response

1. **Confirmation**: We'll work with you to confirm the vulnerability
2. **Assessment**: We'll assess the severity and impact
3. **Fix Development**: We'll develop and test a fix
4. **Disclosure**: We'll coordinate disclosure timing with you
5. **Credit**: We'll credit you in our security advisories (unless you prefer otherwise)

## Security Updates

Security updates will be released as soon as possible after a vulnerability is confirmed. Users will be notified through:

- GitHub security advisories
- Release notes
- Documentation updates

## Scope

This security policy applies to:

- The main Claude Night Pilot application (both CLI and GUI)
- All official plugins and extensions
- Documentation and configuration examples

Out of scope:
- Third-party Claude CLI tool security issues
- Issues in dependencies (report to respective maintainers)
- General system administration security

## Contact

For questions about this security policy, please create an issue in the GitHub repository or contact the maintainers directly.

---

**Last Updated**: August 2, 2025  
**Version**: 1.0