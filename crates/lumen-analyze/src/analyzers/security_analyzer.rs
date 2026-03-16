//! Security Analyzer
//!
//! Comprehensive security analysis including:
//! - Vulnerability scanning (dependencies, code patterns)
//! - Secrets detection (API keys, tokens, passwords)
//! - SQL injection detection
//! - XSS vulnerability patterns
//! - CSRF protection checks
//! - Authentication/authorization issues
//! - Insecure configurations

use lumen_core::{LumenResult, Project, ProjectInfo, Framework, Language};
use lumen_score::{IssueSeverity, MetricValue, ScoreIssue};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// Security analyzer
pub struct SecurityAnalyzer {
    project_root: String,
}

/// Secret pattern detected
#[derive(Debug, Clone)]
struct DetectedSecret {
    secret_type: SecretType,
    file: String,
    line: usize,
    value_preview: String,
}

/// Types of secrets that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SecretType {
    ApiKey,
    JwtToken,
    Password,
    AwsAccessKey,
    AwsSecretKey,
    GitHubToken,
    SlackToken,
    DatabaseUrl,
    PrivateKey,
    AuthToken,
    OAuthSecret,
}

impl SecurityAnalyzer {
    /// Create a new security analyzer
    pub fn new(project_root: String) -> Self {
        Self { project_root }
    }

    /// Analyze the project for security issues
    pub fn analyze(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // 1. Secrets detection
        let (secret_metrics, secret_issues) = self.detect_secrets(info)?;
        metrics.extend(secret_metrics);
        issues.extend(secret_issues);

        // 2. SQL injection detection
        let (sqli_metrics, sqli_issues) = self.detect_sql_injection(info)?;
        metrics.extend(sqli_metrics);
        issues.extend(sqli_issues);

        // 3. XSS detection
        let (xss_metrics, xss_issues) = self.detect_xss_patterns(info)?;
        metrics.extend(xss_metrics);
        issues.extend(xss_issues);

        // 4. Dependency vulnerability check
        let (dep_metrics, dep_issues) = self.check_dependency_vulnerabilities(info)?;
        metrics.extend(dep_metrics);
        issues.extend(dep_issues);

        // 5. Authentication/authorization checks
        let (auth_metrics, auth_issues) = self.check_auth_security(info)?;
        metrics.extend(auth_metrics);
        issues.extend(auth_issues);

        // 6. Configuration security
        let (config_metrics, config_issues) = self.check_configuration_security(info)?;
        metrics.extend(config_metrics);
        issues.extend(config_issues);

        // 7. HTTPS/SSL checks
        let (ssl_metrics, ssl_issues) = self.check_ssl_security(info)?;
        metrics.extend(ssl_metrics);
        issues.extend(ssl_issues);

        // 8. Input validation checks
        let (input_metrics, input_issues) = self.check_input_validation(info)?;
        metrics.extend(input_metrics);
        issues.extend(input_issues);

        // Calculate overall security score
        let base_score = 100.0;
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 25.0,
            IssueSeverity::High => 15.0,
            IssueSeverity::Medium => 8.0,
            IssueSeverity::Low => 3.0,
            IssueSeverity::Info => 0.0,
        }).sum();

        metrics.insert(
            "security:overall_score".to_string(),
            MetricValue::Percentage((base_score - penalty).max(0.0)),
        );

        Ok((metrics, issues))
    }

    /// Detect secrets in code
    fn detect_secrets(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let mut detected_secrets = Vec::new();

        // Secret patterns (regex-based detection)
        let patterns = [
            // Generic API keys
            (SecretType::ApiKey, Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['"]([a-zA-Z0-9_\-]{20,})['"]"#).unwrap()),
            // JWT tokens
            (SecretType::JwtToken, Regex::new(r"eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap()),
            // AWS keys
            (SecretType::AwsAccessKey, Regex::new(r#"(?i)aws[_-]?(access[_-]?key[_-]?id|secret[_-]?access[_-]?key)\s*[:=]\s*['"]?[A-Z0-9]{20}['"]?"#).unwrap()),
            // GitHub tokens
            (SecretType::GitHubToken, Regex::new(r#"(?i)github[_-]?(token|pat)\s*[:=]\s*['"](ghp_[a-zA-Z0-9]{36})['"]"#).unwrap()),
            // Database URLs
            (SecretType::DatabaseUrl, Regex::new(r#"(?i)(database[_-]?url|db[_-]?url|mongodb[_-]?uri|postgres[_-]?url)\s*[:=]\s*['"]([^'"]{20,})['"]"#).unwrap()),
            // Private keys
            (SecretType::PrivateKey, Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap()),
            // Auth tokens
            (SecretType::AuthToken, Regex::new(r#"(?i)(auth[_-]?token|bearer[_-]?token|access[_-]?token)\s*[:=]\s*['"]([a-zA-Z0-9_\-]{20,})['"]"#).unwrap()),
            // OAuth secrets
            (SecretType::OAuthSecret, Regex::new(r#"(?i)(client[_-]?secret|oauth[_-]?secret)\s*[:=]\s*['"]([a-zA-Z0-9_\-]{20,})['"]"#).unwrap()),
            // Slack tokens
            (SecretType::SlackToken, Regex::new(r"xox[pbar]-[a-zA-Z0-9-]{10,}").unwrap()),
        ];

        let source_files = self.find_source_files(info)?;

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    for (secret_type, pattern) in &patterns {
                        if let Some(caps) = pattern.captures(line) {
                            // Skip if in example, test, or documentation
                            if self.is_likely_safe_context(file_path, line) {
                                continue;
                            }

                            let value = caps.get(caps.len() - 1).map(|m| m.as_str()).unwrap_or("");
                            detected_secrets.push(DetectedSecret {
                                secret_type: secret_type.clone(),
                                file: file_path.to_string_lossy().to_string(),
                                line: line_idx + 1,
                                value_preview: if value.len() > 10 {
                                    format!("{}...", &value[..10])
                                } else {
                                    value.to_string()
                                },
                            });
                        }
                    }
                }
            }
        }

        metrics.insert(
            "security:secrets_detected".to_string(),
            MetricValue::Count(detected_secrets.len()),
        );

        // Group secrets by type for reporting
        let mut secrets_by_type: HashMap<SecretType, usize> = HashMap::new();
        for secret in &detected_secrets {
            *secrets_by_type.entry(secret.secret_type.clone()).or_insert(0) += 1;
        }

        for (secret_type, count) in secrets_by_type {
            let type_name = format!("{:?}", secret_type);
            issues.push(ScoreIssue {
                id: format!("sec-{:?}", secret_type).to_lowercase(),
                severity: IssueSeverity::Critical,
                category: "security".to_string(),
                title: format!("Potential {:?} detected", secret_type),
                description: format!("Found {} potential {} exposed in code. Secrets should be stored in environment variables.", count, type_name.to_lowercase()),
                file: None,
                line: None,
                column: None,
                impact: 25.0 * count as f64,
                suggestion: Some(format!("Move secrets to environment variables:\n\n```bash\n# .env file (add to .gitignore!)\n{}_KEY=your_secret_here\n\n# Or use a secrets manager:\n# - AWS Secrets Manager\n# - HashiCorp Vault\n# - Azure Key Vault\n```\n\n```typescript\n// Access via environment\nconst apiKey = process.env.{}_KEY;\n```", type_name.to_uppercase().replace('_', "_"), type_name.to_uppercase().replace('_', "_"))),
            });
        }

        Ok((metrics, issues))
    }

    /// Check if a line is in a safe context (test, example, documentation)
    fn is_likely_safe_context(&self, file_path: &Path, line: &str) -> bool {
        let path_str = file_path.to_string_lossy();
        let is_test_file = path_str.contains("test") || path_str.contains("spec") || path_str.contains("__tests__");
        let is_example_file = path_str.contains("example") || path_str.contains("demo") || path_str.contains("sample");
        let is_doc_file = path_str.ends_with(".md") || path_str.ends_with(".txt") || path_str.contains("docs");

        let line_indicates_example = line.contains("example")
            || line.contains("EXAMPLE")
            || line.contains("your-api-key")
            || line.contains("YOUR_")
            || line.contains("<insert>");

        is_test_file || is_example_file || is_doc_file || line_indicates_example
    }

    /// Detect SQL injection patterns
    fn detect_sql_injection(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let source_files = self.find_source_files(info)?;

        // SQL injection patterns
        let dangerous_patterns = [
            // String concatenation in queries
            (Regex::new(r##"query\s*[\+=]\s*['"][^'"]*\$\{[^}]+\}[^'"]*['"]"##).unwrap(), "Template literal with variable interpolation in SQL query"),
            (Regex::new(r##"query\s*\+=\s*['"][^'"']*\+\s*\w+[^'"']*['"]"##).unwrap(), "String concatenation in SQL query"),
            // Direct variable interpolation
            (Regex::new(r##"(execute|query|raw)\s*\(\s*['"][^'"]*\+[^'"]*['"]"##).unwrap(), "Variable interpolation in SQL execution"),
            // Format strings with user input
            (Regex::new(r#"format\s*\([^)]*\+[^)]*\)"#).unwrap(), "String formatting in query"),
        ];

        let mut sqli_count = 0;
        let mut sqli_locations = Vec::new();

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    for (pattern, description) in &dangerous_patterns {
                        if pattern.is_match(line) {
                            sqli_count += 1;
                            sqli_locations.push((file_path.to_string_lossy().to_string(), line_idx + 1, description.to_string()));
                        }
                    }
                }
            }
        }

        metrics.insert(
            "security:sql_injection_risks".to_string(),
            MetricValue::Count(sqli_count),
        );

        if sqli_count > 0 {
            issues.push(ScoreIssue {
                id: "sec-001".to_string(),
                severity: IssueSeverity::Critical,
                category: "security".to_string(),
                title: "Potential SQL injection vulnerability".to_string(),
                description: format!("Found {} potential SQL injection vulnerabilities. User input is being directly concatenated into SQL queries.", sqli_count),
                file: sqli_locations.first().map(|l| Some(l.0.clone())).flatten(),
                line: sqli_locations.first().map(|l| Some(l.1)).flatten(),
                column: None,
                impact: 25.0 * sqli_count as f64,
                suggestion: Some("Use parameterized queries or prepared statements:\n\n```typescript\n// ❌ VULNERABLE:\nconst query = `SELECT * FROM users WHERE id = ${userId}`;\ndb.execute(query);\n\n// ✅ SECURE:\nconst query = 'SELECT * FROM users WHERE id = $1';\ndb.execute(query, [userId]);\n\n// Or with ORM:\nconst user = await User.findByPk(userId);\n```\n\n```rust\n// ❌ VULNERABLE:\nlet query = format!(\"SELECT * FROM users WHERE id = '{}', user_id);\n\n// ✅ SECURE:\nlet user = sqlx::query_as::<_, User>(\"SELECT * FROM users WHERE id = $1\")\n    .bind(&user_id)\n    .fetch_one(&pool)\n    .await?;\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Detect XSS vulnerability patterns
    fn detect_xss_patterns(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Only relevant for web frameworks
        if !matches!(info.framework,
            Framework::NextJs | Framework::ViteReact | Framework::ViteVue | Framework::ViteSvelte |
            Framework::SvelteKit | Framework::Nuxt | Framework::Remix | Framework::Astro | Framework::Express |
            Framework::NestJS | Framework::Fastify)
        {
            return Ok((metrics, issues));
        }

        let source_files = self.find_source_files(info)?;

        // XSS patterns
        let xss_patterns = [
            // dangerouslySetInnerHTML in React
            (Regex::new(r#"dangerouslySetInnerHTML\s*=\s*\{\s*\{\s*__html\s*:\s*[^}]+\}\s*\}"#).unwrap(), "dangerouslySetInnerHTML with user input"),
            // v-html in Vue
            (Regex::new(r#"v-html\s*=\s*[\"'][^\"']*\$\{[^}]+\}[^\"]*['\"]"#).unwrap(), "v-html with user input"),
            // innerHTML assignment
            (Regex::new(r#"\.innerHTML\s*=\s*[^;]+"#).unwrap(), "innerHTML assignment with user input"),
            // document.write with user input
            (Regex::new(r#"document\.write\s*\(\s*[^)]*\+[^)]*\)"#).unwrap(), "document.write with concatenated content"),
        ];

        let mut xss_count = 0;

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    for (pattern, description) in &xss_patterns {
                        if pattern.is_match(line) && !line.contains("/* safe */") && !line.contains("// safe") {
                            xss_count += 1;
                            if xss_count <= 5 { // Limit reported locations
                                issues.push(ScoreIssue {
                                    id: "sec-002".to_string(),
                                    severity: IssueSeverity::High,
                                    category: "security".to_string(),
                                    title: "Potential XSS vulnerability".to_string(),
                                    description: format!("{} allows rendering untrusted HTML, which can lead to XSS attacks.", description),
                                    file: Some(file_path.to_string_lossy().to_string()),
                                    line: Some(line_idx + 1),
                                    column: None,
                                    impact: 15.0,
                                    suggestion: Some("Sanitize HTML before rendering:\n\n```tsx\n// ❌ VULNERABLE:\n<div dangerouslySetInnerHTML={{ __html: userContent }} />\n\n// ✅ SECURE - Use DOMPurify:\nimport DOMPurify from 'dompurify';\n<div dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(userContent) }} />\n\n// ✅ SECURE - Use a library:\nimport { sanitize } from 'sanitize-html';\n<div dangerouslySetInnerHTML={{ __html: sanitize(userContent) }} />\n\n// ✅ SECURE - Use React's built-in escaping:\n<div>{userContent}</div>\n```".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }

        metrics.insert(
            "security:xss_risks".to_string(),
            MetricValue::Count(xss_count),
        );

        Ok((metrics, issues))
    }

    /// Check for known vulnerable dependencies
    fn check_dependency_vulnerabilities(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Known vulnerable versions/patterns (this would normally use a security advisory database)
        let known_vulnerable = [
            ("express", "4.17.3", "CVE-2023-26806 - Express DoS vulnerability"),
            ("lodash", "4.17.20", "CVE-2021-23337 - Prototype pollution"),
            ("axios", "0.21.1", "CVE-2021-3749 - Server-Side Request Forgery"),
            ("node-forge", "1.3.0", "CVE-2022-0122 - RSA PKCS#1 signature verification"),
            ("ua-parser-js", "0.7.28", "CVE-2022-25923 - ReDoS vulnerability"),
        ];

        let mut vulnerable_count = 0;

        for dep in &info.dependencies {
            for (vuln_pkg, vuln_ver, description) in &known_vulnerable {
                if dep.contains(vuln_pkg) {
                    vulnerable_count += 1;
                    issues.push(ScoreIssue {
                        id: format!("sec-dep-{}", vuln_pkg.replace('-', "")),
                        severity: IssueSeverity::High,
                        category: "security".to_string(),
                        title: format!("Known vulnerable dependency: {}", vuln_pkg),
                        description: format!("Package '{}' has known vulnerabilities: {}", vuln_pkg, description),
                        file: None,
                        line: None,
                        column: None,
                        impact: 15.0,
                        suggestion: Some(format!("Update to the latest secure version:\n\n```bash\nnpm update {}\n```\n\nAlways run: npm audit and npm audit fix", vuln_pkg)),
                    });
                }
            }
        }

        // Check if package-lock.json / yarn.lock exists (indicates dependencies are locked)
        let root = Path::new(&self.project_root);
        let has_lock_file = root.join("package-lock.json").exists()
            || root.join("yarn.lock").exists()
            || root.join("pnpm-lock.yaml").exists()
            || root.join("Cargo.lock").exists();

        metrics.insert(
            "security:has_lock_file".to_string(),
            MetricValue::Boolean(has_lock_file),
        );

        if !has_lock_file {
            issues.push(ScoreIssue {
                id: "sec-003".to_string(),
                severity: IssueSeverity::Medium,
                category: "security".to_string(),
                title: "No lock file detected".to_string(),
                description: "Lock files ensure consistent dependency versions across installations and help prevent supply chain attacks.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Commit your lock files to version control:\n\n```bash\n# npm\npackage-lock.json\n\n# Yarn\nyarn.lock\n\n# pnpm\npnpm-lock.yaml\n\n# Rust\nCargo.lock\n```\n\nRun `npm install` or `cargo build` to generate the lock file.".to_string()),
            });
        }

        metrics.insert(
            "security:vulnerable_dependencies".to_string(),
            MetricValue::Count(vulnerable_count),
        );

        Ok((metrics, issues))
    }

    /// Check authentication/authorization security
    fn check_auth_security(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let source_files = self.find_source_files(info)?;

        // Check for hardcoded credentials
        let hardcoded_creds = Regex::new(r#"(?i)(password|passwd|pwd|secret)\s*[:=]\s*['\"](?!env|\$|\$\{)([a-zA-Z0-9@#$%^&*]{8,})['"]"#).unwrap();

        let mut hardcoded_count = 0;

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    if hardcoded_creds.is_match(line) && !self.is_likely_safe_context(file_path, line) {
                        hardcoded_count += 1;
                        if hardcoded_count <= 3 {
                            issues.push(ScoreIssue {
                                id: "sec-004".to_string(),
                                severity: IssueSeverity::Critical,
                                category: "security".to_string(),
                                title: "Hardcoded credentials detected".to_string(),
                                description: "Hardcoded passwords or secrets in source code can be exploited by attackers.".to_string(),
                                file: Some(file_path.to_string_lossy().to_string()),
                                line: Some(line_idx + 1),
                                column: None,
                                impact: 20.0,
                                suggestion: Some("Move credentials to environment variables:\n\n```typescript\n// ❌ BAD:\nconst password = 'SuperSecret123';\n\n// ✅ GOOD:\nconst password = process.env.DB_PASSWORD;\n```\n\n```rust\n// ❌ BAD:\nlet password = \"SuperSecret123\";\n\n// ✅ GOOD:\nlet password = std::env::var(\"DB_PASSWORD\").expect(\"DB_PASSWORD must be set\");\n```".to_string()),
                            });
                        }
                    }
                }
            }
        }

        // Check for JWT without verification
        let jwt_without_verify = Regex::new(r#"jwt\.verify|verify\(|jwt.*verify"#).unwrap();
        let jwt_decode_only = Regex::new(r#"jwt\.decode|decode\(|jwt.*decode"#).unwrap();

        let mut has_verify = false;
        let mut has_decode_only = false;

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if jwt_without_verify.is_match(&content) {
                    has_verify = true;
                }
                if jwt_decode_only.is_match(&content) {
                    has_decode_only = true;
                }
            }
        }

        metrics.insert(
            "security:uses_jwt_verification".to_string(),
            MetricValue::Boolean(has_verify),
        );

        if has_decode_only && !has_verify {
            issues.push(ScoreIssue {
                id: "sec-005".to_string(),
                severity: IssueSeverity::High,
                category: "security".to_string(),
                title: "JWT decoded without verification".to_string(),
                description: "JWT tokens are being decoded but their signature is not being verified, allowing token forgery attacks.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 15.0,
                suggestion: Some("Always verify JWT signatures:\n\n```typescript\n// ❌ BAD:\nconst decoded = jwt.decode(token);\n\n// ✅ GOOD:\nconst decoded = jwt.verify(token, process.env.JWT_SECRET);\n```\n\n```rust\n// ❌ BAD:\nlet claims = decode_jwt(token);\n\n// ✅ GOOD:\nlet claims = decode_jwt::<Claims>(&token, &Validation::new(&DecodingKey::from_secret(&secret)))?;\n```".to_string()),
            });
        }

        metrics.insert(
            "security:hardcoded_credentials".to_string(),
            MetricValue::Count(hardcoded_count),
        );

        Ok((metrics, issues))
    }

    /// Check configuration security
    fn check_configuration_security(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);

        // Check for .env file committed
        let env_files = [
            root.join(".env"),
            root.join(".env.local"),
            root.join(".env.production"),
        ];

        let has_env_in_git = env_files.iter().any(|p| p.exists());

        if has_env_in_git {
            issues.push(ScoreIssue {
                id: "sec-006".to_string(),
                severity: IssueSeverity::Critical,
                category: "security".to_string(),
                title: "Environment file may be committed to git".to_string(),
                description: ".env files containing secrets should not be committed to version control.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 25.0,
                suggestion: Some("Add .env files to .gitignore:\n\n```\n# Environment files\n.env\n.env.local\n.env.*.local\n```\n\nCreate a .env.example file with placeholder values:\n\n```\nDATABASE_URL=postgresql://user:password@localhost:5432/dbname\nJWT_SECRET=your-secret-key-here\nAPI_KEY=your-api-key-here\n```".to_string()),
            });
        }

        // Check .gitignore for sensitive files
        let gitignore = root.join(".gitignore");
        let has_proper_gitignore = if gitignore.exists() {
            if let Ok(content) = std::fs::read_to_string(&gitignore) {
                content.contains(".env")
                    || content.contains("*.key")
                    || content.contains("*.pem")
            } else {
                false
            }
        } else {
            false
        };

        metrics.insert(
            "security:has_gitignore".to_string(),
            MetricValue::Boolean(has_proper_gitignore),
        );

        if !has_proper_gitignore {
            issues.push(ScoreIssue {
                id: "sec-007".to_string(),
                severity: IssueSeverity::Medium,
                category: "security".to_string(),
                title: "Missing or incomplete .gitignore".to_string(),
                description: "Sensitive files may be committed to version control without proper .gitignore rules.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 8.0,
                suggestion: Some("Create a comprehensive .gitignore:\n\n```\n# Dependencies\nnode_modules/\ntarget/\ndist/\n\n# Environment files\n.env\n.env.local\n.env.*.local\n\n# Secrets\n*.key\n*.pem\n*.cert\n*.crt\nsecrets/\n\n# IDE\n.idea/\n.vscode/\n*.swp\n\n# OS\n.DS_Store\nThumbs.db\n```".to_string()),
            });
        }

        // Check for CORS misconfiguration
        let cors_wildcard = Regex::new(r##"(?i)origin\s*:\s*['"]\*['"]|cors\(\s*\{\s*origin\s*:\s*['"]\*['"]"##).unwrap();

        for file_path in self.find_source_files(info)? {
            let file_path_str = file_path.to_string_lossy().to_string();
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if cors_wildcard.is_match(&content) {
                    issues.push(ScoreIssue {
                        id: "sec-008".to_string(),
                        severity: IssueSeverity::Medium,
                        category: "security".to_string(),
                        title: "Overly permissive CORS configuration".to_string(),
                        description: "CORS is configured to allow requests from any origin (\"*\"). This can expose your API to malicious websites.".to_string(),
                        file: Some(file_path_str),
                        line: None,
                        column: None,
                        impact: 8.0,
                        suggestion: Some("Configure specific allowed origins:\n\n```typescript\n// ❌ BAD:\napp.use(cors({ origin: '*' }));\n\n// ✅ GOOD:\napp.use(cors({\n  origin: [\n    'https://yourdomain.com',\n    'https://app.yourdomain.com',\n  ],\n  credentials: true,\n}));\n\n// Or dynamic:\napp.use(cors({\n  origin: function(origin, callback) {\n    const allowedOrigins = ['https://yourdomain.com'];\n    if (!origin || allowedOrigins.includes(origin)) {\n      callback(null, true);\n    } else {\n      callback(new Error('Not allowed by CORS'));\n    }\n  },\n}));\n```".to_string()),
                    });
                    break;
                }
            }
        }

        Ok((metrics, issues))
    }

    /// Check SSL/HTTPS configuration
    fn check_ssl_security(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for http:// URLs in configuration
        let insecure_url = Regex::new(r#"http://(?!localhost|127\.0\.0\.1|0\.0\.0\.0)[^s\"]"#).unwrap();

        for file_path in self.find_source_files(info)? {
            let file_path_str = file_path.to_string_lossy().to_string();
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                for (line_idx, line) in content.lines().enumerate() {
                    if insecure_url.is_match(line) {
                        issues.push(ScoreIssue {
                            id: "sec-009".to_string(),
                            severity: IssueSeverity::Medium,
                            category: "security".to_string(),
                            title: "Insecure HTTP URL detected".to_string(),
                            description: "HTTP URLs transmit data in plain text, allowing interception.".to_string(),
                            file: Some(file_path_str),
                            line: Some(line_idx + 1),
                            column: None,
                            impact: 5.0,
                            suggestion: Some("Use HTTPS for all external connections:\n\n```typescript\n// ❌ BAD:\nconst apiUrl = 'http://api.example.com/data';\n\n// ✅ GOOD:\nconst apiUrl = 'https://api.example.com/data';\n```".to_string()),
                        });
                        break;
                    }
                }
            }
        }

        // Check for cookie security flags
        let has_secure_cookies = self.count_pattern_in_files(info, r"(?i)secure\s*:\s*true|cookie.*secure|sameSite")?;

        metrics.insert(
            "security:has_secure_cookies".to_string(),
            MetricValue::Boolean(has_secure_cookies > 0),
        );

        Ok((metrics, issues))
    }

    /// Check input validation
    fn check_input_validation(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for validation libraries
        let has_validation = info.dependencies.iter().any(|d| {
            d.contains("zod")
                || d.contains("yup")
                || d.contains("joi")
                || d.contains("validator")
                || d.contains("class-validator")
        });

        let has_schema_validation = info.cargo_dependencies.as_ref()
            .map(|deps| deps.iter().any(|(k, _)| {
                k.contains("validator") || k.contains("schema") || k.contains("validate")
            }))
            .unwrap_or(false);

        metrics.insert(
            "security:has_input_validation".to_string(),
            MetricValue::Boolean(has_validation || has_schema_validation),
        );

        if !has_validation && !has_schema_validation && matches!(info.framework, Framework::NestJS | Framework::Express | Framework::Fastify | Framework::NextJs) {
            issues.push(ScoreIssue {
                id: "sec-010".to_string(),
                severity: IssueSeverity::Medium,
                category: "security".to_string(),
                title: "No input validation library detected".to_string(),
                description: "Input validation is critical for preventing injection attacks, data corruption, and other security issues.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 8.0,
                suggestion: Some("Add an input validation library:\n\n```bash\n# TypeScript/JavaScript\nnpm install zod\n# or\nnpm install yup\n\n# Rust\ncargo add validator\n```\n\n```typescript\n// Using Zod\nimport { z } from 'zod';\n\nconst UserSchema = z.object({\n  email: z.string().email(),\n  password: z.string().min(8),\n  age: z.number().min(0).max(120),\n});\n\nfunction validateUser(data: unknown) {\n  return UserSchema.parse(data);\n}\n```".to_string()),
            });
        }

        // Check for rate limiting
        let has_rate_limit = info.dependencies.iter().any(|d| {
            d.contains("rate-limit")
                || d.contains("express-rate-limit")
                || d.contains("rate-limiter")
                || d.contains("throttle")
        });

        metrics.insert(
            "security:has_rate_limiting".to_string(),
            MetricValue::Boolean(has_rate_limit),
        );

        if !has_rate_limit && matches!(info.framework, Framework::NestJS | Framework::Express | Framework::Fastify) {
            issues.push(ScoreIssue {
                id: "sec-011".to_string(),
                severity: IssueSeverity::Medium,
                category: "security".to_string(),
                title: "No rate limiting detected".to_string(),
                description: "Rate limiting helps prevent brute force attacks, DDoS, and resource exhaustion.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 8.0,
                suggestion: Some("Add rate limiting:\n\n```bash\nnpm install express-rate-limit\n```\n\n```typescript\nimport rateLimit from 'express-rate-limit';\n\nconst limiter = rateLimit({\n  windowMs: 15 * 60 * 1000, // 15 minutes\n  max: 100, // Limit each IP to 100 requests per windowMs\n  standardHeaders: true,\n  legacyHeaders: false,\n});\n\napp.use('/api', limiter);\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Find all source files in the project
    fn find_source_files(&self, info: &ProjectInfo) -> LumenResult<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        let extensions = match info.language {
            Language::TypeScript | Language::JavaScript => &["ts", "tsx", "js", "jsx"][..],
            Language::Rust => &["rs", "toml"][..],
            Language::Python => &["py"][..],
            Language::Go => &["go"][..],
            _ => &["ts", "tsx", "js", "jsx", "rs", "py"],
        };

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        let path_str = path.to_string_lossy();
                        if !path_str.contains("node_modules")
                            && !path_str.contains("target")
                            && !path_str.contains(".next")
                            && !path_str.contains("dist")
                        {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Count pattern occurrences in source files
    fn count_pattern_in_files(&self, info: &ProjectInfo, pattern: &str) -> LumenResult<usize> {
        let regex = Regex::new(pattern).unwrap();
        let mut count = 0;

        let source_files = self.find_source_files(info)?;
        for file in source_files {
            if let Ok(content) = std::fs::read_to_string(&file) {
                count += regex.find_iter(&content).count();
            }
        }

        Ok(count)
    }
}

/// Public function for backward compatibility with the module interface
pub fn analyze(project: &lumen_core::Project) -> Vec<ScoreIssue> {
    let analyzer = SecurityAnalyzer::new(project.info.root.to_string_lossy().to_string());
    let (_, issues) = analyzer.analyze(&project.info).unwrap_or_default();
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_analyzer_creation() {
        let analyzer = SecurityAnalyzer::new("/tmp".to_string());
        assert_eq!(analyzer.project_root, "/tmp");
    }

    #[test]
    fn test_safe_context_detection() {
        let analyzer = SecurityAnalyzer::new("/tmp".to_string());
        let test_file = Path::new("/tmp/example_test.ts");
        let line = "const apiKey = 'example-api-key-for-testing';";

        assert!(analyzer.is_likely_safe_context(test_file, line));
    }
}
