#!/usr/bin/env bash
# Script pour publier tous les crates LumenX sur crates.io dans l'ordre des dépendances

set -e  # Arrêter en cas d'erreur

echo "🚀 Publishing LumenX v0.5.2 crates to crates.io"
echo "================================================"

# Ordre de publication basé sur les dépendances
CRATES=(
    "lumenx-core"
    "lumenx-detect"
    "lumenx-score"
    "lumenx-report"
    "lumenx-testgen"
    "lumenx-analyze"
    "lumenx-fix"
    "lumenx-cli"
)

# Fonction pour publier un crate
publish_crate() {
    local crate=$1
    local crate_path="crates/$crate"

    echo ""
    echo "📦 Publishing $crate..."
    echo "--------------------------"

    # Dry-run d'abord
    echo "🔍 Dry-run..."
    if cargo publish --dry-run -p "$crate"; then
        echo "✅ Dry-run successful for $crate"

        # Publication réelle
        echo "🚀 Publishing..."
        cd "$crate_path"
        cargo publish -p "$crate"
        cd ../..

        echo "✅ $crate published successfully!"
    else
        echo "❌ Dry-run failed for $crate, skipping..."
        return 1
    fi
}

# Publier chaque crate dans l'ordre
for crate in "${CRATES[@]}"; do
    if ! publish_crate "$crate"; then
        echo ""
        echo "❌ Failed to publish $crate"
        echo "   Fix the issue and re-run the script"
        echo "   Already published crates:"
        for published in "${CRATES[@]}"; do
            if [ "$published" = "$crate" ]; then
                break
            fi
            echo "   - $published"
        done
        exit 1
    fi

    # Attendre un peu que crates.io indexe le package
    echo "⏳ Waiting for crates.io to index..."
    sleep 5
done

echo ""
echo "================================================"
echo "🎉 All crates published successfully!"
echo ""
echo "Users can now install with:"
echo "  cargo install lumenx-cli"
