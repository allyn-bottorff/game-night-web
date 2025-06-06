// Authentication handling
document.addEventListener('DOMContentLoaded', () => {
    // DOM elements
    const authContainer = document.getElementById('auth-container');
    const mainContent = document.getElementById('main-content');
    const usernameDisplay = document.getElementById('username-display');
    const logoutBtn = document.getElementById('logout-btn');
    const adminTab = document.getElementById('admin-tab');
    
    // Tab switching
    const loginTab = document.getElementById('login-tab');
    const registerTab = document.getElementById('register-tab');
    const loginForm = document.getElementById('login-form');
    const registerForm = document.getElementById('register-form');
    
    // Login form
    const loginUsername = document.getElementById('login-username');
    const loginPassword = document.getElementById('login-password');
    const loginError = document.getElementById('login-error');
    
    // Register form
    const registerUsername = document.getElementById('register-username');
    const registerPassword = document.getElementById('register-password');
    const registerError = document.getElementById('register-error');
    
    // Check if user is already logged in
    checkAuthStatus();
    
    // Tab switching logic
    loginTab.addEventListener('click', () => {
        loginTab.classList.add('tab-active');
        registerTab.classList.remove('tab-active');
        loginForm.style.display = 'block';
        registerForm.style.display = 'none';
    });
    
    registerTab.addEventListener('click', () => {
        registerTab.classList.add('tab-active');
        loginTab.classList.remove('tab-active');
        registerForm.style.display = 'block';
        loginForm.style.display = 'none';
    });
    
    // Login form submission
    loginForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        loginError.textContent = '';
        
        try {
            const response = await api.login(loginUsername.value, loginPassword.value);
            handleLoginSuccess(response);
        } catch (error) {
            loginError.textContent = error.data?.error || 'Login failed. Please try again.';
        }
    });
    
    // Register form submission
    registerForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        registerError.textContent = '';
        
        try {
            const response = await api.register(registerUsername.value, registerPassword.value);
            handleLoginSuccess(response);
        } catch (error) {
            registerError.textContent = error.data?.error || 'Registration failed. Please try again.';
        }
    });
    
    // Logout button
    logoutBtn.addEventListener('click', () => {
        localStorage.removeItem('auth_token');
        localStorage.removeItem('user');
        checkAuthStatus();
    });
    
    // Check authentication status
    function checkAuthStatus() {
        const token = localStorage.getItem('auth_token');
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        
        if (token && user) {
            // User is logged in
            authContainer.style.display = 'none';
            mainContent.style.display = 'block';
            usernameDisplay.textContent = user.username;
            logoutBtn.style.display = 'inline-block';
            
            // Show admin tab if user is admin
            if (user.is_admin) {
                adminTab.style.display = 'block';
            } else {
                adminTab.style.display = 'none';
            }
        } else {
            // User is not logged in
            authContainer.style.display = 'block';
            mainContent.style.display = 'none';
            usernameDisplay.textContent = 'Not logged in';
            logoutBtn.style.display = 'none';
            
            // Clear forms
            loginForm.reset();
            registerForm.reset();
            loginError.textContent = '';
            registerError.textContent = '';
        }
    }
    
    // Handle successful login/registration
    function handleLoginSuccess(response) {
        localStorage.setItem('auth_token', response.token);
        localStorage.setItem('user', JSON.stringify(response.user));
        checkAuthStatus();
    }
});

// Expose these for other modules to use
function getCurrentUser() {
    return JSON.parse(localStorage.getItem('user') || 'null');
}

function isAuthenticated() {
    return !!localStorage.getItem('auth_token');
}

function isAdmin() {
    const user = getCurrentUser();
    return user && user.is_admin;
}