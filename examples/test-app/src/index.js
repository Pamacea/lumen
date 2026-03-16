// Test file for security and performance analysis

// API key exposure test (should be detected)
const API_KEY = "sk-1234567890abcdefghijklmnopqrstuvwxyz";
const DATABASE_URL = "postgres://user:password@localhost:5432/db";

// Insecure HTTP URL (should NOT be flagged - localhost is OK)
const LOCAL_API = "http://localhost:3000/api";

// Insecure HTTP URL (SHOULD be flagged - not localhost)
const EXTERNAL_API = "http://example.com/api/data";

// Missing await test
async function fetchData() {
    // Fire-and-forget pattern (may be detected)
    const result = fetch("/api/data");

    // Proper await
    const data = await fetch("/api/data");
    return data.json();
}

// Sequential awaits
async function processItems() {
    const item1 = await fetch("/item/1");
    const item2 = await fetch("/item/2");
    const item3 = await fetch("/item/3");
    return [item1, item2, item3];
}

// N+1 query pattern
async function getUsersWithPosts() {
    const users = await db.query("SELECT * FROM users");
    for (const user of users) {
        // N+1 query - should be detected
        user.posts = await db.query("SELECT * FROM posts WHERE user_id = ?", user.id);
    }
    return users;
}

// Export test
module.exports = { fetchData, processItems, getUsersWithPosts };
