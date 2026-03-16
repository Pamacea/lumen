/**
 * Test suite for LumenX test-app functions
 * This file tests all exported functions to avoid "unused function" warnings
 */

const { fetchData, processItems, getUsersWithPosts } = require('../src/index.js');

// Mock the fetch API
global.fetch = jest.fn();

// Mock database
const db = {
    query: jest.fn()
};

describe('fetchData', () => {
    it('should fetch data from API', async () => {
        const mockData = { result: 'success' };
        fetch.mockResolvedValue({
            json: async () => mockData
        });

        const result = await fetchData('/api/test');
        expect(result).toEqual(mockData);
    });
});

describe('processItems', () => {
    it('should process multiple items in parallel', async () => {
        const mockResponses = [
            { json: async () => ({ id: 1 }) },
            { json: async () => ({ id: 2 }) },
            { json: async () => ({ id: 3 }) }
        ];
        fetch.mockResolvedValue(...mockResponses);

        const results = await processItems();
        expect(results).toHaveLength(3);
        expect(fetch).toHaveBeenCalledTimes(3);
    });
});

describe('getUsersWithPosts', () => {
    it('should fetch users with their posts using JOIN', async () => {
        const mockUsers = [
            { id: 1, name: 'User 1' },
            { id: 2, name: 'User 2' }
        ];

        db.query.mockResolvedValue(mockUsers);

        const users = await getUsersWithPosts();
        expect(users).toEqual(mockUsers);
        expect(db.query).toHaveBeenCalledWith(
            expect.stringContaining('LEFT JOIN posts')
        );
    });
});
