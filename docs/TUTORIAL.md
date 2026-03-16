# Lumen - Tutoriel

Guide d'utilisation complet de Lumen v0.5.0, un toolkit d'analyse de code et de génération de tests alimenté par l'IA.

## Installation

### Depuis le code source

```bash
# Cloner le repository
git clone https://github.com/votre-org/lumen.git
cd lumen

# Compiler et installer
cargo install -p lumenx --path .
```

### Avec cargo (une fois publié)

```bash
cargo install lumenx-cli
```

## Première utilisation

### Initialiser un projet

```bash
cd votre-projet
lumenx init
```

Cela crée :
- `lumen.toml` - Configuration personnalisée
- `.lumen/` - Dossier de travail Lumen

### Scanner un projet

```bash
# Analyse complète avec rapports
lumenx scan

# Sortie JSON pour l'intégration CI
lumenx scan --format json --output results.json

# Analyse avec filtres
lumenx scan --dimensions coverage,security --severity high
```

## Commandes principales

### 1. `lumen scan` - Analyse complète

Analyse toutes les dimensions de qualité du code :

```bash
# Usage basique
lumenx scan

# Options courantes
lumenx scan --format html          # Rapport HTML
lumenx scan --format markdown     # Rapport Markdown (défaut)
lumenx scan --output ./reports    # Dossier de sortie personnalisé
```

**Résultat :**
- Score global (0-100)
- Scores par dimension (7 dimensions)
- Liste des problèmes trouvés
- Suggestions d'amélioration

### 2. `lumen detect` - Détection de framework

Détecte automatiquement le stack technique :

```bash
lumenx detect

# Sortie JSON pour les scripts
lumenx detect --json
```

**Détecte :**
- Framework (Next.js, Remix, Vite, Axum, etc.)
- Langage principal
- Test runner
- Package manager
- Dépendances principales

### 3. `lumen analyze` - Analyse ciblée

Analyse spécifique par type :

```bash
# Analyse de sécurité uniquement
lumenx analyze --analyzer security

# Analyse statique
lumenx analyze --analyzer static

# Analyse de performance
lumenx analyze --analyzer performance

# Avec filtrage par sévérité
lumenx analyze --severity high --output findings.json
```

**Analyzers disponibles :**
- `static` - Problèmes de code
- `security` - Vulnérabilités de sécurité
- `dependency` - Dépendances obsolètes/vulnérables
- `performance` - Problèmes de performance
- `seo` - Optimisation SEO
- `uiux` - Problèmes UI/UX
- `docs` - Documentation manquante

### 4. `lumen score` - Scoring uniquement

Calcule le score sans analyse complète :

```bash
lumenx score

# Score détaillé par dimension
lumenx score --detailed

# Comparaison avec l'historique
lumenx score --compare

# Sortie JSON
lumenx score --json
```

### 5. `lumen generate-tests` - Génération de tests

Génère des tests basés sur votre code :

```bash
# Détection automatique du framework
lumenx generate-tests

# Framework spécifique
lumenx generate-tests --framework vitest
lumenx generate-tests --framework jest
lumenx generate-tests --framework pytest

# Mode dry-run (prévisualisation)
lumenx generate-tests --dry-run

# Dossier de sortie personnalisé
lumenx generate-tests --output ./my-tests
```

**Frameworks supportés :**
- `vitest` - TypeScript/JavaScript moderne
- `jest` - React/Node.js
- `cargo` / `nextest` - Rust
- `pytest` - Python

### 6. `lumen fix` - Correction automatique

Corrige automatiquement certains problèmes :

```bash
# Mode dry-run (voir les modifications sans appliquer)
lumenx fix --dry-run

# Mode interactif (confirmer chaque fix)
lumenx fix --interactive

# Appliquer tout automatiquement
lumenx fix --yes

# Filtrer par sévérité et catégorie
lumenx fix --min-severity high --categories security,performance
```

### 7. `lumen report` - Rapports avancés

Génère des rapports multi-formats :

```bash
# Rapport unique (format par défaut: markdown)
lumenx report

# Tous les formats
lumenx report --all

# Format spécifique
lumenx report --format html
lumenx report --format json
lumenx report --format junit

# Avec analyse de tendance
lumenx report --trend
```

## Workflow recommandé

### Développement quotidien

```bash
# 1. Scanner rapide pour voir l'état actuel
lumenx scan

# 2. Corriger les problèmes critiques automatiquement
lumenx fix --min-severity critical --yes

# 3. Analyser les problèmes restants
lumenx analyze --analyzer security --severity high

# 4. Générer des tests pour le nouveau code
lumenx generate-tests --framework vitest
```

### Intégration CI/CD

```bash
# Dans votre pipeline CI
lumenx scan --format json --output lumen-results.json

# Le code de sortie indique si le score est acceptable
if [ $? -ne 0 ]; then
  echo "Score de qualité insuffisant"
  exit 1
fi
```

### Pré-commit hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Vérifier le score minimum
lumenx scan --threshold 70
if [ $? -ne 0 ]; then
  echo "Score de qualité insuffisant (minimum: 70)"
  exit 1
fi

# Linter et tests
cargo test --all
```

## Configuration

### Fichier `lumen.toml`

```toml
[general]
verbose = false
quiet = false
no_color = false

[scoring.weights]
coverage = 0.25    # Couverture de tests
quality = 0.20     # Qualité du code
performance = 0.15 # Performance
security = 0.15    # Sécurité
seo = 0.10         # SEO (pour les sites web)
docs = 0.05        # Documentation
uiux = 0.10        # UI/UX

[scoring.thresholds]
excellent = 80.0
good = 60.0

[analysis]
static_analysis = true
security = true
dependencies = true
performance = true
seo = true
uiux = true

[report]
output_dir = "./lumen-reports"
formats = ["md", "json"]

[analysis.exclude]
paths = ["node_modules", "target", "dist", "build", ".git", "vendor"]
```

### Options CLI

```bash
# Options globales
lumenx --help                    # Aide
lumenx --version                 # Version
lumenx --verbose                 # Verbose
lumenx --quiet                   # Silencieux
lumenx --no-color                # Pas de couleurs

# Commutateurs utiles
lumenx scan -q                   # Mode silencieux
lumenx scan -v                   # Mode verbose
lumenx scan --format json -o -   # Sortie stdout JSON
```

## Exemples d'utilisation

### Projet Rust

```bash
# Analyser un projet Rust
cd my-rust-project
lumenx scan

# Vérifier la couverture
lumenx analyze --analyzer static

# Générer des tests
lumenx generate-tests --framework cargo

# Corriger les problèmes simples
lumenx fix --dry-run  # Voir d'abord
lumenx fix --yes       # Appliquer
```

### Projet Next.js

```bash
# Analyser un projet Next.js
cd my-nextjs-app
lumenx scan

# Focus SEO et performance
lumenx analyze --analyzer seo,performance --severity medium

# Générer tests Vitest
lumenx generate-tests --framework vitest --output src/__tests__
```

### Monorepo

```bash
# Analyser tout le monorepo
lumenx scan --path ./monorepo

# Analyser un package spécifique
lumenx scan --path ./monorepo/packages/backend

# Comparer les packages
lumenx score --path ./packages/frontend
lumenx score --path ./packages/backend
```

## Interprétation des scores

### Score global

| Score | Grade | Signification |
|-------|-------|---------------|
| 90-100 | A+ | Excellent |
| 80-89 | A | Très bon |
| 70-79 | B | Bon |
| 60-69 | C | Acceptable |
| 50-59 | D | À améliorer |
| 0-49 | F | Critique |

### Dimensions

1. **Coverage (25%)** - Couverture de tests
2. **Quality (20%)** - Qualité du code
3. **Performance (15%)** - Performance
4. **Security (15%)** - Sécurité
5. **SEO (10%)** - SEO (web)
6. **Documentation (5%)** - Documentation
7. **UI/UX (10%)** - Interface utilisateur

## Dépannage

### Erreurs courantes

```bash
# "Path does not exist"
# → Vérifiez que le chemin est correct
lumenx scan --path ./correct-path

# "No files scanned"
# → Vérifiez que vous avez des fichiers source dans le projet

# "Permission denied"
# → Vérifiez les permissions sur le dossier .lumen/
```

### Mode verbose

```bash
# Activer le debug
LUMEN_LOG=debug lumen scan --verbose
```

### Nettoyer le cache

```bash
# Supprimer le dossier de travail
rm -rf .lumen/

# Réinitialiser
lumenx init --defaults
```

## Ressources

- Documentation API : `docs/API.md`
- Architecture : `docs/ARCHITECTURE.md`
- GitHub Issues : https://github.com/votre-org/lumen/issues
- Discord : https://discord.gg/lumen
