// API configuration
const API_BASE_URL = 'http://localhost:3000/api';

// API helper for making requests
const api = {
    // Authentication API
    async login(username, password) {
        return this.post('/login', { username, password });
    },

    async register(username, password) {
        return this.post('/register', { username, password });
    },

    // Polls API
    async getActivePolls() {
        return this.get('/polls?active=true');
    },

    async getInactivePolls() {
        return this.get('/polls?active=false');
    },

    async getPoll(pollId) {
        return this.get(`/polls/${pollId}`);
    },

    async createPoll(pollData) {
        return this.post('/polls', pollData);
    },

    async votePoll(pollId, optionId) {
        return this.post(`/polls/${pollId}/vote`, { option_id: optionId });
    },

    async getPollResults(pollId) {
        return this.get(`/polls/${pollId}/results`);
    },

    // Users API (admin)
    async listUsers() {
        return this.get('/users');
    },

    async createUser(userData) {
        return this.post('/users', userData);
    },

    // Base methods
    async get(endpoint) {
        const response = await fetch(`${API_BASE_URL}${endpoint}`, {
            method: 'GET',
            headers: this.getHeaders()
        });
        return this.handleResponse(response);
    },

    async post(endpoint, data) {
        const response = await fetch(`${API_BASE_URL}${endpoint}`, {
            method: 'POST',
            headers: this.getHeaders(),
            body: JSON.stringify(data)
        });
        return this.handleResponse(response);
    },

    async put(endpoint, data) {
        const response = await fetch(`${API_BASE_URL}${endpoint}`, {
            method: 'PUT',
            headers: this.getHeaders(),
            body: JSON.stringify(data)
        });
        return this.handleResponse(response);
    },

    async delete(endpoint) {
        const response = await fetch(`${API_BASE_URL}${endpoint}`, {
            method: 'DELETE',
            headers: this.getHeaders()
        });
        return this.handleResponse(response);
    },

    // Helper methods
    getHeaders() {
        const headers = {
            'Content-Type': 'application/json',
        };
        
        const token = localStorage.getItem('auth_token');
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }
        
        return headers;
    },

    async handleResponse(response) {
        const data = await response.json();
        
        if (!response.ok) {
            const error = new Error(data.error || 'API Error');
            error.status = response.status;
            error.data = data;
            throw error;
        }
        
        return data;
    }
};