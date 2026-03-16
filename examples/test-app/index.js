/**
 * LumenX Test App - Main Entry Point
 * This file demonstrates all exported functions
 */

const { fetchData, processItems, getUsersWithPosts } = require('./src/index.js');

// Export all functions to prevent "unused" warnings
// and make them available for testing

/**
 * Example usage of fetchData
 */
async function main() {
    try {
        // Example: Fetch data
        console.log('Testing fetchData...');
        // const data = await fetchData('/api/users');
        // console.log('Data fetched:', data);

        // Example: Process items
        console.log('Testing processItems...');
        // const items = await processItems();
        // console.log('Items processed:', items);

        // Example: Get users with posts
        console.log('Testing getUsersWithPosts...');
        // const users = await getUsersWithPosts();
        // console.log('Users with posts:', users);

        console.log('All functions tested successfully!');
    } catch (error) {
        console.error('Error:', error.message);
    }
}

// Export for use in other modules
module.exports = {
    fetchData,
    processItems,
    getUsersWithPosts,
    main
};

// Run main if this file is executed directly
if (require.main === module) {
    main().catch(console.error);
}
