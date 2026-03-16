# Lumen - Architecture

Documentation architecturale de Lumen v0.5.0, un toolkit d'analyse de code et de génération de tests en Rust.

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────────┐
│                           Lumen CLI                             │
│                         (crates/lumen)                          │
└─────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────────┼───────────────────────────┐
        │           │               │           │               │
        ▼           ▼               ▼           ▼               ▼
┌──────────┐ ┌──────────┐   ┌──────────┐ ┌──────────┐   ┌──────────┐
│  Detect  │ │ Analyze  │   │  Score   │ │TestGen   │   │   Fix    │
└──────────┘ └──────────┘   └──────────┘ └──────────┘   └──────────┘
      │             │               │           │               │
      └─────────────┴───────────────┴───────────┴───────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   lumen-core     │
                    │  (Types & Utils) │
                    └──────────────────┘
```

## Structure du workspace

```
lumen/
├── Cargo.toml                 # Workspace root (manifest virtuel)
├── Cargo.lock                 # Lock file partagé
│
├── crates/
│   ├── lumen/                 # Binaire CLI
│   │   ├── src/
│   │   │   ├── main.rs        # Point d'entrée
│   │   │   └── cli.rs         # Définition des commandes CLI
│   │   └── Cargo.toml
│   │
│   ├── lumen-core/            # Types et utilitaires partagés
│   │   ├── src/
│   │   │   ├── lib.rs         # Point d'entrée
│   │   │   ├── project.rs     # Structure Project
│   │   │   ├── language.rs    # Énumération Language
│   │   │   ├── framework.rs   # Énumération Framework
│   │   │   └── error.rs       # Types d'erreur
│   │   └── Cargo.toml
│   │
│   ├── lumen-detect/          # Détection automatique
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── detector.rs    # FrameworkDetector
│   │   │   ├── patterns/      # Patterns de détection
│   │   │   └── heuristics.rs  # Heuristiques
│   │   └── Cargo.toml
│   │
│   ├── lumen-analyze/         # Moteur d'analyse
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── analyzer.rs    # Analyzer principal
│   │   │   ├── analyzers/
│   │   │   │   ├── static_analyzer.rs
│   │   │   │   ├── security_analyzer.rs
│   │   │   │   ├── dependency_analyzer.rs
│   │   │   │   ├── performance_analyzer.rs
│   │   │   │   ├── seo_analyzer.rs
│   │   │   │   ├── uiux_analyzer.rs
│   │   │   │   └── docs_analyzer.rs
│   │   │   ├── ast/
│   │   │   │   ├── parser.rs   # Tree-sitter wrapper
│   │   │   │   └── query.rs    # Requêtes AST
│   │   │   └── parsers/
│   │   │       ├── typescript.rs
│   │   │       ├── rust.rs
│   │   │       ├── python.rs
│   │   │       └── html.rs
│   │   └── Cargo.toml
│   │
│   ├── lumen-score/           # Système de scoring
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── calculator.rs  # ScoreCalculator
│   │   │   ├── dimensions.rs  # DimensionScores
│   │   │   ├── grade.rs       # Grade (A+/A/B/C/D/F)
│   │   │   ├── metrics.rs     # MetricValue
│   │   │   ├── trend.rs       # TrendAnalysis
│   │   │   ├── history.rs     # ScoreHistory
│   │   │   └── types.rs       # Types réexportés
│   │   └── Cargo.toml
│   │
│   ├── lumen-testgen/         # Génération de tests
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── generator.rs   # TestGenerator
│   │   │   ├── code_parser.rs # Parse le code source
│   │   │   ├── templates/
│   │   │   │   ├── typescript.rs
│   │   │   │   ├── rust.rs
│   │   │   │   ├── python.rs
│   │   │   │   └── mod.rs
│   │   │   └── types.rs       # Types de test
│   │   └── Cargo.toml
│   │
│   ├── lumen-fix/             # Corrections automatiques
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── engine.rs      # FixEngine
│   │   │   ├── fixer.rs       # AutoFixer
│   │   │   ├── fixers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── security.rs
│   │   │   │   ├── quality.rs
│   │   │   │   ├── performance.rs
│   │   │   │   └── style.rs
│   │   │   └── patch.rs       # Gestion des patches
│   │   └── Cargo.toml
│   │
│   ├── lumen-report/          # Génération de rapports
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── generator.rs   # ReportGenerator
│   │   │   ├── formats/
│   │   │   │   ├── markdown.rs
│   │   │   │   ├── json.rs
│   │   │   │   ├── html.rs
│   │   │   │   └── junit.rs
│   │   │   └── templates/     # Templates de rapport
│   │   └── Cargo.toml
│   │
│   └── lumen-detect/          # (répété, cf. ci-dessus)
│
├── docs/                      # Documentation
│   ├── TUTORIAL.md
│   ├── ARCHITECTURE.md
│   └── API.md
│
└── README.md                  # README principal
```

## Composants détaillés

### 1. lumen-core

**Responsabilité :** Types et utilitaires partagés

```rust
// Types principaux
pub struct Project {
    pub info: ProjectInfo,
    pub source_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
    pub config_files: Vec<PathBuf>,
    pub total_lines: usize,
    pub coverage: Option<f64>,
}

pub struct ProjectInfo {
    pub name: String,
    pub framework: Framework,
    pub language: Language,
    pub test_runner: TestRunner,
    pub package_manager: Option<String>,
    pub dependencies: Vec<String>,
}

// Énumérations
pub enum Language {
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Go,
    // ...
}

pub enum Framework {
    NextJs,
    Remix,
    ViteReact,
    Axum,
    ActixWeb,
    // ...
}
```

**Dépendances :**
- `serde` - Sérialisation
- `thiserror` - Erreurs
- `tracing` - Logging

### 2. lumen-detect

**Responsabilité :** Détection automatique du stack technique

```
┌─────────────────────────────────────────┐
│         FrameworkDetector              │
├─────────────────────────────────────────┤
│  detect() -> ProjectInfo                │
│    ├── detect_language()                │
│    ├── detect_framework()               │
│    ├── detect_test_runner()             │
│    ├── detect_package_manager()         │
│    └── parse_dependencies()             │
└─────────────────────────────────────────┘
```

**Patterns de détection :**
- Fichiers de configuration (`package.json`, `Cargo.toml`, etc.)
- Structure de dossiers
- Dépendances installées
- Patterns de code

### 3. lumen-analyze

**Responsabilité :** Analyse statique et dynamique du code

```
┌─────────────────────────────────────────┐
│              Analyzer                   │
├─────────────────────────────────────────┤
│  analyze() -> AnalysisResult            │
│    ├── StaticAnalyzer                   │
│    ├── SecurityAnalyzer                 │
│    ├── DependencyAnalyzer               │
│    ├── PerformanceAnalyzer              │
│    ├── SeoAnalyzer                      │
│    ├── UiUxAnalyzer                     │
│    └── DocsAnalyzer                     │
└─────────────────────────────────────────┘
```

**AST Parsing avec Tree-sitter :**

```rust
pub struct AstParser {
    language: AstLanguage,
    parser: tree_sitter::Parser,
}

impl AstParser {
    pub fn parse(&self, source: &str) -> Tree {
        // Parse avec tree-sitter
    }

    pub fn query(&self, pattern: &str) -> Vec<Node> {
        // Requêtes AST
    }
}
```

### 4. lumen-score

**Responsabilité :** Calcul des scores de qualité

```
┌─────────────────────────────────────────┐
│         ScoreCalculator                 │
├─────────────────────────────────────────┤
│  calculate(project, metrics) -> Score   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │     DimensionScores (7 dims)    │   │
│  ├─────────────────────────────────┤   │
│  │  • Coverage (25%)               │   │
│  │  • Quality (20%)                │   │
│  │  • Performance (15%)            │   │
│  │  • Security (15%)               │   │
│  │  • SEO (10%)                    │   │
│  │  • Documentation (5%)           │   │
│  │  • UI/UX (10%)                  │   │
│  └─────────────────────────────────┘   │
│                                         │
│  Overall = Σ(Dimension × Weight)        │
│  Grade = from_score(Overall)            │
└─────────────────────────────────────────┘
```

**Système de grades :**

```rust
pub enum Grade {
    APlus,  // 90-100
    A,      // 85-89
    AMinus, // 80-84
    BPlus,  // 75-79
    B,      // 70-74
    BMinus, // 65-69
    CPlus,  // 60-64
    C,      // 55-59
    CMinus, // 50-54
    DPlus,  // 45-49
    D,      // 40-44
    F,      // 0-39
}
```

### 5. lumen-testgen

**Responsabilité :** Génération automatique de tests

```
┌─────────────────────────────────────────┐
│         TestGenerator                  │
├─────────────────────────────────────────┤
│  generate(project) -> Tests             │
│    ├── CodeParser (extrait fonctions)  │
│    ├── TemplateRenderer                │
│    └── FrameworkAdapter                │
└─────────────────────────────────────────┘
```

**Workflow de génération :**

1. **Parsing** - Extrait les fonctions/signatures du code source
2. **Analyse** - Identifie les paramètres, types, et comportements
3. **Génération** - Applique le template approprié
4. **Validation** - Vérifie que les tests compilent

### 6. lumen-fix

**Responsabilité :** Corrections automatiques

```
┌─────────────────────────────────────────┐
│           FixEngine                     │
├─────────────────────────────────────────┤
│  apply_fixes(issues) -> FixResult       │
│    ├── SecurityFixer                    │
│    ├── QualityFixer                     │
│    ├── PerformanceFixer                 │
│    └── StyleFixer                       │
└─────────────────────────────────────────┘
```

**Système de patch :**

```rust
pub struct Patch {
    pub file: String,
    pub issue_id: String,
    pub description: String,
    pub hunks: Vec<PatchHunk>,
}

pub struct PatchHunk {
    pub old_range: (usize, usize),
    pub new_range: (usize, usize),
    pub remove: Vec<String>,
    pub add: Vec<String>,
}
```

### 7. lumen-report

**Responsabilité :** Génération de rapports multi-formats

```
┌─────────────────────────────────────────┐
│       ReportGenerator                   │
├─────────────────────────────────────────┤
│  generate(score, format) -> Report      │
│    ├── MarkdownRenderer                 │
│    ├── JsonRenderer                     │
│    ├── HtmlRenderer                     │
│    └── JunitRenderer                    │
└─────────────────────────────────────────┘
```

## Flux de données

### Commande `lumen scan`

```
1. CLI parse les arguments
   └─> cmd_scan()

2. Detect le framework
   └─> FrameworkDetector::detect()
       └─> ProjectInfo

3. Scanne les fichiers
   └─> scan_source_files(), scan_test_files()
       └─> Vec<PathBuf>

4. Analyse le code
   └─> Analyzer::new(project).analyze()
       └─> AnalysisResult {
             static_findings,
             security_findings,
             // ...
         }

5. Calcule les scores
   └─> ScoreCalculator::calculate()
       └─> ProjectScore

6. Génère les rapports
   └─> ReportGenerator::generate()
       └─> Vec<Report>
```

### Commande `luman fix`

```
1. Analyse les problèmes
   └─> Analyzer::analyze()
       └─> Vec<ScoreIssue>

2. Filtre les problèmes fixables
   └─> filter(|i| i.suggestion.is_some())

3. Applique les corrections
   └─> FixEngine::apply_fixes()
       ├── Pour chaque issue
       │   ├── Fixer approprié
       │   ├── Génère le patch
       │   ├── Applique (si !dry_run)
       │   └── Git rollback (si demandé)
       └─> FixResult {
             fixed: Vec<SuccessFix>,
             failed: Vec<FailedFix>,
         }
```

## Design patterns utilisés

### 1. Builder Pattern

```rust
let engine = FixEngine::new(path)
    .with_dry_run()
    .with_branch()
    .with_tests();
```

### 2. Trait Strategy

```rust
trait Fixer {
    type Fix;

    fn can_fix(&self, issue: &ScoreIssue) -> bool;
    fn fix(&self, content: &str, issue: &ScoreIssue) -> Result<(String, Self::Fix)>;
}

// Implémentations
struct SecurityFixer { ... }
struct QualityFixer { ... }
```

### 3. Visitor Pattern (AST)

```rust
trait AstVisitor {
    fn visit_function(&mut self, fn: &FunctionInfo);
    fn visit_class(&mut self, class: &ClassInfo);
    fn visit_import(&mut self, import: &ImportInfo);
}
```

## Performance

### Optimisations

1. **Parallélisme** - Analyse multi-fichiers avec `rayon`
2. **Cache** - Résultats d'analyse en cache
3. **Incrémental** - Analyse uniquement les fichiers modifiés
4. **Lazy parsing** - AST parsing à la demande

### Benchmarks

| Projet | Fichiers | LOC | Temps scan |
|--------|----------|-----|------------|
| Small (<100 files) | ~50 | 5K | ~2s |
| Medium (100-500) | ~250 | 25K | ~8s |
| Large (500+) | ~1000 | 100K | ~30s |

## Sécurité

### Analyse locale

Tout le code s'exécute localement - aucune donnée n'est envoyée à un serveur externe.

### Validation d'entrée

- Validation des chemins de fichiers
- Limitation de la taille des fichiers analysés
- Timeout sur les opérations de parsing

### Corrections automatiques

- Mode dry-run par défaut
- Validation des patches avant application
- Git rollback automatique en cas d'erreur

## Extensibilité

### Ajouter un nouvel analyzer

```rust
// 1. Créer le module
pub struct MyCustomAnalyzer {
    project: Project,
}

impl MyCustomAnalyzer {
    pub fn analyze(&self) -> Vec<ScoreIssue> {
        // Implémentation
    }
}

// 2. Enregistrer dans lumen-analyze/src/lib.rs
pub mod my_custom_analyzer;

// 3. Ajouter à Analyzer::analyze()
```

### Ajouter un nouveau format de rapport

```rust
// 1. Créer le renderer
pub struct MyCustomRenderer;

impl ReportRenderer for MyCustomRenderer {
    fn render(&self, score: &ProjectScore) -> String {
        // Implémentation
    }
}

// 2. Enregistrer dans ReportFormat
```

## Dépendances externes

| Crate | Usage |
|-------|------|
| `tree-sitter` | AST parsing |
| `clap` | CLI parsing |
| `tokio` | Async runtime |
| `regex` | Pattern matching |
| `walkdir` | File system traversal |
| `serde` | Serialization |
| `chrono` | Dates/heures |
| `colored` | Terminal colors |
| `indicatif` | Progress bars |

## Tests

### Structure des tests

```
crates/
├── lumen-core/tests/
│   ├── project_tests.rs
│   └── error_tests.rs
├── lumen-analyze/tests/
│   ├── analyzer_tests.rs
│   └── security_tests.rs
├── lumen-score/tests/
│   ├── calculator_tests.rs
│   └── grade_tests.rs
└── lumen-fix/tests/
    ├── fixer_tests.rs
    └── patch_tests.rs
```

### Exécuter les tests

```bash
# Tous les tests
cargo test --workspace

# Tests spécifiques
cargo test -p lumen-score
cargo test --test score_calculator

# Avec output
cargo test -- --nocapture --test-threads=1
```

## Contribuer

### Setup de développement

```bash
# Cloner
git clone https://github.com/votre-org/lumen.git
cd lumen

# Formatter
cargo install rustfmt
rustfmt --all

# Linter
cargo install clippy
cargo clippy --all-targets --all-features

# Tests
cargo test --workspace
```

### Convention de code

- `rustfmt` pour le formatage
- `clippy` pour le linting
- Documentation sur tous les types publics
- Tests unitaires pour la logique métier

### Workflow de PR

1. Forker le projet
2. Créer une branche `feature/ma-fonctionnalite`
3. Commiter avec des messages clairs
4. Pusher et créer une PR
5. Attendre la review CI

## Licence

MIT License - voir LICENSE pour plus de détails.
