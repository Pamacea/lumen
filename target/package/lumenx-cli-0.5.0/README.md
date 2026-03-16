# Lumen

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/lumen)](https://crates.io/crates/lumen)
[![docs.rs](https://img.shields.io/docsrs/lumen)](https://docs.rs/lumen)

> AI-powered code analysis and test generation toolkit - 100% Rust

**Lumen** analyzes your codebase, generates quality reports, and provides AI-ready fix suggestions for automated code improvement.

## Features

- 🔍 **Framework Detection** - Auto-detects Next.js, Rust, NestJS, Remix, SvelteKit, and more
- 📊 **7-Dimension Scoring** - Coverage, Quality, Performance, Security, SEO, Docs, UI/UX
- 🧪 **Test Generation** - Framework-specific test templates
- 🎯 **AI-Ready Reports** - `report.md` for humans, `fixes.md` for AI agents
- ⚡ **Blazing Fast** - Written in Rust for maximum performance
- 🛠️ **cargo install** - Single binary installation

## Quick Start

```bash
# Install
cargo install lumen-cli

# Analyze your project
lumen scan

# Get AI-ready fixes
lumen scan --output ./reports
cat ./reports/fixes.md  # Feed this to Claude Code, Cursor, etc.
```

## Scoring Dimensions

| Dimension | Weight | Metrics |
|-----------|--------|---------|
| **Coverage** | 25% | Unit, integration, E2E tests |
| **Quality** | 20% | Complexity, duplication, lint |
| **Performance** | 15% | Backend latency, Core Web Vitals |
| **Security** | 15% | Vulnerabilities, insecure code |
| **SEO** | 10% | Meta tags, Open Graph, structured data |
| **Docs** | 5% | README, API docs, comments |
| **UI/UX** | 10% | Layout, responsive, accessibility, design |

## Output Example

```
╔═══════════════════════════════════════════════════════════╗
║                    LUMEN REPORT v0.5.0                   ║
╠═══════════════════════════════════════════════════════════╣
║                                                            ║
║  Framework: Next.js 14                                     ║
║  Language: TypeScript                                      ║
║  Score: 82/100 (B)                                        ║
║                                                            ║
║  ┌────────────────────────────────────────────────────┐   ║
║  │  QUALITY SCORE                                      │   ║
║  │  Overall: 82/100 (B)                               │   ║
║  │                                                     │   ║
║  │  Coverage    ████████████████░░ 85/100            │   ║
║  │  Quality     ██████████████░░░░ 78/100            │   ║
║  │  Performance ████████████████░░ 88/100            │   ║
║  │  Security    ████████████░░░░░░ 75/100            │   ║
║  │  SEO         ████████████████░░ 82/100            │   ║
║  │  Docs        ██████████░░░░░░░░ 70/100            │   ║
║  │  UI/UX       ██████████████░░░░ 80/100            │   ║
║  └────────────────────────────────────────────────────┘   ║
║                                                            ║
║  Issues: 🔴 3 Critical | 🟠 8 High | 🟡 15 Medium         ║
║  Fixable: 26 | Auto-fix available: --fix flag             ║
╚═══════════════════════════════════════════════════════════╝
```

## Commands

```bash
lumen scan                    # Full analysis with AI-ready fixes
lumen init                    # Initialize config
lumen detect                  # Detect framework and tools
lumen analyze                  # Analyze code only
lumen score                    # Show quality scores
lumen generate-tests           # Generate test templates
lumen fix                      # Apply automatic fixes
lumen report --format=html     # Generate reports
lumen history                  # View score trends
```

## Supported Frameworks

### Frontend
- Next.js, Remix, SvelteKit, Nuxt, Astro
- Vite + React/Vue/Svelte
- Angular, Solid

### Backend
- **Rust**: Axum, Actix, Rocket, Poem
- **Node.js**: Express, Fastify, NestJS
- **Python**: (planned)

## Installation

```bash
# From crates.io
cargo install lumen

# Or build from source
cargo install --path .
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Scoring System](docs/SCORING.md)
- [UI/UX Analyzer](docs/UIUX.md)
- [Contributing](docs/CONTRIBUTING.md)

## License

MIT - See [LICENSE](LICENSE) for details.

## Credits

Built for modern development teams who care about code quality.
Inspired by [Daemon](https://github.com/Oalacea/daemon).
