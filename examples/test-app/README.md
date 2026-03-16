# Test App for LumenX Validation

This is a test application to validate LumenX analysis works correctly.

## Expected Findings

### Security
- ✅ API key exposure: `sk-1234567890...`
- ✅ Database URL with password
- ✅ Insecure HTTP URL to external domain (http://example.com)
- ❌ localhost URLs should NOT be flagged

### Performance
- ✅ Fire-and-forget pattern (missing await)
- ✅ Sequential await chains
- ✅ N+1 query pattern in loop

### Accessibility
- ✅ Images without alt text (2 images)
- ✅ Form inputs without labels
- ✅ Missing skip link

### SEO
- ✅ Has meta description
- ✅ Has semantic HTML (header, nav, main, footer)
- ✅ Has proper heading structure
