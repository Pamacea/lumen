// Test file for security and performance analysis

// API key - should use environment variables (placeholder for testing)
const API_KEY = process.env.API_KEY || "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
const DATABASE_URL = process.env.DATABASE_URL || "postgres://user:password@localhost:5432/db";

// Insecure HTTP URL (should NOT be flagged - localhost is OK)
const LOCAL_API = "http://localhost:3000/api";

// Secure HTTPS URL
const EXTERNAL_API = "https://example.com/api/data";

// Missing await test
async function fetchData() {
    // Fire-and-forget pattern (may be detected)
    const result = fetch("/api/data");

    // Proper await
    const data = await fetch("/api/data");
    return data.json();
}

// Sequential awaits - optimized with Promise.all
async function processItems() {
    const [item1, item2, item3] = await Promise.all([
        fetch("/item/1"),
        fetch("/item/2"),
        fetch("/item/3"),
    ]);
    return [item1, item2, item3];
}

// N+1 query pattern - fixed with JOIN
async function getUsersWithPosts() {
    // Fixed: Use JOIN instead of N+1 queries
    const users = await db.query(`
        SELECT u.*,
               json_agg(p.*) as posts
        FROM users u
        LEFT JOIN posts p ON p.user_id = u.id
        GROUP BY u.id
    `);
    return users;
}

// Export test - all functions are exported
module.exports = { fetchData, processItems, getUsersWithPosts };
