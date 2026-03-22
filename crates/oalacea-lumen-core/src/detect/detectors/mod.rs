//! Framework-specific detectors

use std::path::Path;

/// Check if project is Next.js
pub fn check_nextjs(root: &Path) -> bool {
    root.join("next.config.js").exists()
        || root.join("next.config.ts").exists()
        || root.join("next.config.mjs").exists()
        || package_has_dep(root, "next")
}

/// Check if project is NestJS
pub fn check_nestjs(root: &Path) -> bool {
    package_has_dep(root, "@nestjs/core")
        || package_has_dep(root, "@nestjs/common")
}

/// Check if project is Axum (Rust)
pub fn check_axum(root: &Path) -> bool {
    cargo_has_dep(root, "axum")
}

/// Check if project is Actix Web (Rust)
pub fn check_actix(root: &Path) -> bool {
    cargo_has_dep(root, "actix-web")
}

/// Check if project is Rocket (Rust)
pub fn check_rocket(root: &Path) -> bool {
    cargo_has_dep(root, "rocket")
}

/// Check if project is Remix
pub fn check_remix(root: &Path) -> bool {
    package_has_dep(root, "@remix-run/node")
        || package_has_dep(root, "@remix-run/react")
}

/// Check if project is SvelteKit
pub fn check_sveltekit(root: &Path) -> bool {
    root.join("svelte.config.js").exists()
        || package_has_dep(root, "@sveltejs/kit")
}

/// Check if project is Nuxt
pub fn check_nuxt(root: &Path) -> bool {
    root.join("nuxt.config.js").exists()
        || root.join("nuxt.config.ts").exists()
        || package_has_dep(root, "nuxt")
}

/// Check if project is Astro
pub fn check_astro(root: &Path) -> bool {
    root.join("astro.config.js").exists()
        || root.join("astro.config.mjs").exists()
        || package_has_dep(root, "astro")
}

/// Check if project is Express
pub fn check_express(root: &Path) -> bool {
    package_has_dep(root, "express")
}

/// Check if project is Fastify
pub fn check_fastify(root: &Path) -> bool {
    package_has_dep(root, "fastify")
}

/// Check if project uses Vite + React
pub fn check_vite_react(root: &Path) -> bool {
    root.join("vite.config.js").exists()
        && package_has_dep(root, "react")
}

/// Check if project uses Vite + Vue
pub fn check_vite_vue(root: &Path) -> bool {
    root.join("vite.config.js").exists()
        && package_has_dep(root, "vue")
}

/// Check if project uses Vite + Svelte
pub fn check_vite_svelte(root: &Path) -> bool {
    root.join("vite.config.js").exists()
        && package_has_dep(root, "svelte")
}

/// Helper: Check if package.json has a dependency
pub fn package_has_dep(root: &Path, dep_name: &str) -> bool {
    let pkg_path = root.join("package.json");
    if !pkg_path.exists() {
        return false;
    }
    
    if let Ok(content) = std::fs::read_to_string(&pkg_path) {
        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
            let deps = pkg.get("dependencies")
                .and_then(|d| d.as_object())
                .or_else(|| pkg.get("devDependencies")
                    .and_then(|d| d.as_object()));
            
            if let Some(deps) = deps {
                return deps.contains_key(dep_name);
            }
        }
    }
    
    false
}

/// Helper: Check if Cargo.toml has a dependency
fn cargo_has_dep(root: &Path, dep_name: &str) -> bool {
    let cargo_path = root.join("Cargo.toml");
    if !cargo_path.exists() {
        return false;
    }
    
    if let Ok(content) = std::fs::read_to_string(&cargo_path) {
        if let Ok(cargo) = toml::from_str::<toml::Value>(&content) {
            if let Some(deps) = cargo.get("dependencies")
                .and_then(|d| d.as_table()) {
                return deps.contains_key(dep_name);
            }
        }
    }
    
    false
}

/// Check if project is Poem (Rust)
pub fn check_poem(root: &Path) -> bool {
    cargo_has_dep(root, "poem")
}

/// Check if project uses Dioxus (Rust GUI)
pub fn check_dioxus(root: &Path) -> bool {
    cargo_has_dep(root, "dioxus") || cargo_has_dep(root, "dioxus-cli")
}

/// Check if project uses Leptos (Rust web framework)
pub fn check_leptos(root: &Path) -> bool {
    cargo_has_dep(root, "leptos") || cargo_has_dep(root, "leptos_dom")
}

/// Check if project uses Tauri (Rust desktop)
pub fn check_tauri(root: &Path) -> bool {
    cargo_has_dep(root, "tauri") || cargo_has_dep(root, "tauri-build")
}

/// Check if project uses Yew (Rust web framework)
pub fn check_yew(root: &Path) -> bool {
    cargo_has_dep(root, "yew")
}

/// Check if project uses Seed (Rust web framework)
pub fn check_seed(root: &Path) -> bool {
    cargo_has_dep(root, "seed")
}

/// Check if project uses Tokio (async runtime)
pub fn check_tokio(root: &Path) -> bool {
    cargo_has_dep(root, "tokio") || cargo_has_dep(root, "tokio-runtime")
}
