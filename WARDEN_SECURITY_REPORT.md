# Warden Security Scan Report

> **AI Agent Instructions:** This report is structured for systematic security fixes.
> Each finding includes file path, line numbers, and actionable recommendations.

**Target:** .
**Scan Time:** 2026-03-22T19:12:09.996915+00:00
**Warden Version:** 0.8.3

## 📊 Scan Summary

| Severity | Count | Priority |
|----------|-------|----------|
| 🔴 CRITICAL | 0 | Fix Immediately |
| 🔴 HIGH | 0 | Fix Soon |
| 🟡 MEDIUM | 0 | Fix Priority |
| 🔵 LOW | 26 | Fix When Possible |
| ⚪ INFO | 2 | Review |
| **Total** | **28** |

## 📁 Files to Fix

🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\static_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\static_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\testgen\cache.rs`](.\crates\oalacea-lumen\src\testgen\cache.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\parsers\css.rs`](.\crates\oalacea-lumen-analysis\src\analyze\parsers\css.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\parsers\html.rs`](.\crates\oalacea-lumen-analysis\src\analyze\parsers\html.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\parsers\python.rs`](.\crates\oalacea-lumen-analysis\src\analyze\parsers\python.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\performance_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\performance_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\fix\fixers\performance.rs`](.\crates\oalacea-lumen\src\fix\fixers\performance.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-core\src\scoring\history.rs`](.\crates\oalacea-lumen-core\src\scoring\history.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\report\formats\markdown.rs`](.\crates\oalacea-lumen\src\report\formats\markdown.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\fix\transformers.rs`](.\crates\oalacea-lumen\src\fix\transformers.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\report\formats\json.rs`](.\crates\oalacea-lumen\src\report\formats\json.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\testgen\parallel_analyzer.rs`](.\crates\oalacea-lumen\src\testgen\parallel_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\testgen\templates\nestjs.rs`](.\crates\oalacea-lumen\src\testgen\templates\nestjs.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\ast\query.rs`](.\crates\oalacea-lumen-analysis\src\analyze\ast\query.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\parsers\typescript.rs`](.\crates\oalacea-lumen-analysis\src\analyze\parsers\typescript.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\fix\git.rs`](.\crates\oalacea-lumen\src\fix\git.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\fix\fixers\imports.rs`](.\crates\oalacea-lumen\src\fix\fixers\imports.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\uiux_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\uiux_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs`](.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs) - 2 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\security_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\security_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\fix\fixers\security.rs`](.\crates\oalacea-lumen\src\fix\fixers\security.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\testgen\code_parser.rs`](.\crates\oalacea-lumen\src\testgen\code_parser.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\ast\traversal.rs`](.\crates\oalacea-lumen-analysis\src\analyze\ast\traversal.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\docs_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\docs_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen-analysis\src\analyze\analyzers\seo_analyzer.rs`](.\crates\oalacea-lumen-analysis\src\analyze\analyzers\seo_analyzer.rs) - 1 issue(s)
🟡 [`.\crates\oalacea-lumen\src\testgen\templates\rust.rs`](.\crates\oalacea-lumen\src\testgen\templates\rust.rs) - 2 issue(s)

## 🔍 Detailed Findings

### 🔵 1. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\fix\fixers\imports.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\fix\fixers\imports.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\fix\fixers\imports.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 2. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\fix\fixers\performance.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\fix\fixers\performance.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\fix\fixers\performance.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 3. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\fix\fixers\security.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\fix\fixers\security.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\fix\fixers\security.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 4. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\fix\git.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\fix\git.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\fix\git.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 5. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\fix\transformers.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\fix\transformers.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\fix\transformers.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 6. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\report\formats\json.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\report\formats\json.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\report\formats\json.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 7. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\report\formats\markdown.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\report\formats\markdown.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\report\formats\markdown.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 8. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\testgen\cache.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\cache.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\testgen\cache.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 9. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\testgen\code_parser.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\code_parser.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\testgen\code_parser.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 10. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\testgen\parallel_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\parallel_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\testgen\parallel_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 11. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\testgen\templates\nestjs.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\templates\nestjs.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\testgen\templates\nestjs.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 12. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen\src\testgen\templates\rust.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\templates\rust.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen\src\testgen\templates\rust.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 13. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\docs_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\docs_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\docs_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 14. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\performance_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\performance_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\performance_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 15. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\security_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\security_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\security_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 16. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\seo_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\seo_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\seo_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 17. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\static_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\static_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\static_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 18. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\uiux_analyzer.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\analyzers\uiux_analyzer.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\analyzers\uiux_analyzer.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 19. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\ast\query.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\ast\query.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\ast\query.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 20. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\ast\traversal.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\ast\traversal.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\ast\traversal.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 21. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\css.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\css.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\parsers\css.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 22. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\html.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\html.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\parsers\html.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 23. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\python.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\python.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\parsers\python.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 24. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 25. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\typescript.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\typescript.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-analysis\src\analyze\parsers\typescript.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### 🔵 26. Potential panic with unwrap/expect

**Priority:** 📝 **LOW** - Fix when possible

**Location:** `.\crates\oalacea-lumen-core\src\scoring\history.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-core\src\scoring\history.rs`

**Description:**

File contains unwrap/expect calls: .\crates\oalacea-lumen-core\src\scoring\history.rs

**Fix:**

Consider using pattern matching or ? operator

**CWE:** `CWE-720`

---

### ⚪ 27. Unsafe Rust code detected (1 occurrences)

**Priority:** ℹ️ **INFO** - Review

**Location:** `.\crates\oalacea-lumen\src\testgen\templates\rust.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen\src\testgen\templates\rust.rs`

**Description:**

File contains unsafe Rust blocks: .\crates\oalacea-lumen\src\testgen\templates\rust.rs

**Fix:**

Review unsafe code for memory safety issues

**CWE:** `CWE-119`

---

### ⚪ 28. Unsafe Rust code detected (15 occurrences)

**Priority:** ℹ️ **INFO** - Review

**Location:** `.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs`

> **AI:** Use `Read` tool to view `.\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs`

**Description:**

File contains unsafe Rust blocks: .\crates\oalacea-lumen-analysis\src\analyze\parsers\rust.rs

**Fix:**

Review unsafe code for memory safety issues

**CWE:** `CWE-119`

---

## 🤖 Suggested Fix Order


*Generated by [Warden](https://github.com/Pamacea/warden) v0.8.3* - AI-Readable Report
