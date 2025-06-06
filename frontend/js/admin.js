// Admin functionality
document.addEventListener('DOMContentLoaded', () => {
    // DOM elements
    const adminTab = document.getElementById('admin-tab');
    const adminContent = document.getElementById('admin-content');
    
    const addUserForm = document.getElementById('add-user-form');
    const newUsername = document.getElementById('new-username');
    const newPassword = document.getElementById('new-password');
    const isAdmin = document.getElementById('is-admin');
    const addUserError = document.getElementById('add-user-error');
    const addUserSuccess = document.getElementById('add-user-success');
    
    const usersContainer = document.getElementById('users-container');
    const usersLoading = document.getElementById('users-loading');
    
    // Show admin tab content when clicked
    adminTab.addEventListener('click', () => {
        // Hide all tab contents
        document.querySelectorAll('.tab-content').forEach(content => {
            content.style.display = 'none';
        });
        
        // Deactivate all tabs
        document.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('tab-active');
        });
        
        // Activate admin tab
        adminTab.classList.add('tab-active');
        adminContent.style.display = 'block';
        
        // Load users
        loadUsers();
    });
    
    // Add user form submission
    addUserForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        addUserError.textContent = '';
        addUserSuccess.textContent = '';
        
        // Validate input
        if (!newUsername.value.trim() || !newPassword.value.trim()) {
            addUserError.textContent = 'Username and password are required';
            return;
        }
        
        try {
            // Create user data
            const userData = {
                username: newUsername.value.trim(),
                password: newPassword.value.trim(),
                is_admin: isAdmin.checked
            };
            
            // Call API to create user
            await api.createUser(userData);
            
            // Clear form
            addUserForm.reset();
            
            // Show success message
            addUserSuccess.textContent = 'User created successfully!';
            
            // Reload users list
            loadUsers();
            
        } catch (error) {
            addUserError.textContent = error.data?.error || 'Failed to create user. Please try again.';
        }
    });
    
    // Load users list
    async function loadUsers() {
        try {
            usersLoading.style.display = 'block';
            usersContainer.innerHTML = '';
            
            const users = await api.listUsers();
            
            if (users.length === 0) {
                usersContainer.innerHTML = '<p>No users found.</p>';
                return;
            }
            
            users.forEach(user => {
                const userElement = document.createElement('div');
                userElement.className = 'user-item';
                
                userElement.innerHTML = `
                    <div class="user-info-detail">
                        <span class="user-name">${escapeHtml(user.username)}
                            ${user.is_admin ? '<span class="admin-badge">Admin</span>' : ''}
                        </span>
                        <span class="user-created">Created: ${formatDate(new Date(user.created_at))}</span>
                    </div>
                `;
                
                usersContainer.appendChild(userElement);
            });
            
        } catch (error) {
            usersContainer.innerHTML = `<p class="error-message">Error loading users: ${error.message}</p>`;
        } finally {
            usersLoading.style.display = 'none';
        }
    }
    
    // Helper functions
    function formatDate(date) {
        return new Intl.DateTimeFormat('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        }).format(date);
    }
    
    function escapeHtml(unsafe) {
        if (!unsafe) return '';
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    }
});