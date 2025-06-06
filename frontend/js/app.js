// Main application logic
document.addEventListener('DOMContentLoaded', () => {
    // Initialize the application
    
    // Function to check if user is authenticated and redirect accordingly
    function checkAuth() {
        const token = localStorage.getItem('auth_token');
        const user = JSON.parse(localStorage.getItem('user') || 'null');
        
        if (!token || !user) {
            // Show auth container, hide main content
            document.getElementById('auth-container').style.display = 'block';
            document.getElementById('main-content').style.display = 'none';
        } else {
            // Hide auth container, show main content
            document.getElementById('auth-container').style.display = 'none';
            document.getElementById('main-content').style.display = 'block';
            
            // Update user display
            document.getElementById('username-display').textContent = user.username;
            document.getElementById('logout-btn').style.display = 'inline-block';
            
            // Show/hide admin tab based on user role
            document.getElementById('admin-tab').style.display = user.is_admin ? 'block' : 'none';
        }
    }
    
    // Check authentication on page load
    checkAuth();
    
    // Set up tab navigation
    const tabs = {
        'active-polls-tab': 'active-polls-content',
        'closed-polls-tab': 'closed-polls-content',
        'create-poll-tab': 'create-poll-content',
        'admin-tab': 'admin-content'
    };
    
    // Add click event listeners to all tabs
    Object.keys(tabs).forEach(tabId => {
        const tab = document.getElementById(tabId);
        if (tab) {
            tab.addEventListener('click', () => {
                // Hide all tab contents
                Object.values(tabs).forEach(contentId => {
                    document.getElementById(contentId).style.display = 'none';
                });
                
                // Show selected tab content
                document.getElementById(tabs[tabId]).style.display = 'block';
                
                // Update active tab styling
                Object.keys(tabs).forEach(id => {
                    document.getElementById(id).classList.remove('tab-active');
                });
                tab.classList.add('tab-active');
            });
        }
    });
    
    // Global error handling for fetch requests
    window.addEventListener('unhandledrejection', event => {
        // Check if it's an authentication error
        if (event.reason && event.reason.status === 401) {
            // Clear local storage and redirect to login
            localStorage.removeItem('auth_token');
            localStorage.removeItem('user');
            checkAuth();
            
            // Show error message
            alert('Your session has expired. Please log in again.');
        }
    });
});