// Polls handling
document.addEventListener('DOMContentLoaded', () => {
    // DOM elements
    const activePollsTab = document.getElementById('active-polls-tab');
    const closedPollsTab = document.getElementById('closed-polls-tab');
    const createPollTab = document.getElementById('create-poll-tab');
    
    const activePollsContent = document.getElementById('active-polls-content');
    const closedPollsContent = document.getElementById('closed-polls-content');
    const createPollContent = document.getElementById('create-poll-content');
    
    const activePollsContainer = document.getElementById('active-polls-container');
    const closedPollsContainer = document.getElementById('closed-polls-container');
    const activePollsLoading = document.getElementById('active-polls-loading');
    const closedPollsLoading = document.getElementById('closed-polls-loading');
    
    // Poll modal elements
    const pollModal = document.getElementById('poll-modal');
    const modalClose = document.querySelector('.close');
    const modalPollTitle = document.getElementById('modal-poll-title');
    const modalPollDescription = document.getElementById('modal-poll-description');
    const modalPollCreator = document.getElementById('modal-poll-creator');
    const modalPollCreated = document.getElementById('modal-poll-created');
    const modalPollExpiryContainer = document.getElementById('modal-poll-expiry-container');
    const modalPollExpiry = document.getElementById('modal-poll-expiry');
    const modalPollOptions = document.getElementById('modal-poll-options');
    const voteForm = document.getElementById('vote-form');
    const voteError = document.getElementById('vote-error');
    const voteSuccess = document.getElementById('vote-success');
    const pollResultsContainer = document.getElementById('poll-results-container');
    const resultsLoading = document.getElementById('results-loading');
    
    // Create poll form elements
    const createPollForm = document.getElementById('create-poll-form');
    const pollTitle = document.getElementById('poll-title');
    const pollDescription = document.getElementById('poll-description');
    const pollExpiry = document.getElementById('poll-expiry');
    const pollOptionsContainer = document.getElementById('poll-options-container');
    const addOptionBtn = document.getElementById('add-option-btn');
    const createPollError = document.getElementById('create-poll-error');
    const createPollSuccess = document.getElementById('create-poll-success');
    
    // Tab switching logic
    activePollsTab.addEventListener('click', () => {
        showTab(activePollsTab, activePollsContent);
        loadActivePolls();
    });
    
    closedPollsTab.addEventListener('click', () => {
        showTab(closedPollsTab, closedPollsContent);
        loadClosedPolls();
    });
    
    createPollTab.addEventListener('click', () => {
        showTab(createPollTab, createPollContent);
    });
    
    function showTab(tab, content) {
        // Deactivate all tabs
        [activePollsTab, closedPollsTab, createPollTab].forEach(t => t.classList.remove('tab-active'));
        [activePollsContent, closedPollsContent, createPollContent].forEach(c => c.style.display = 'none');
        
        // Activate selected tab
        tab.classList.add('tab-active');
        content.style.display = 'block';
    }
    
    // Load polls on page load
    loadActivePolls();
    
    // Load active polls
    async function loadActivePolls() {
        try {
            activePollsLoading.style.display = 'block';
            activePollsContainer.innerHTML = '';
            
            const polls = await api.getActivePolls();
            
            if (polls.length === 0) {
                activePollsContainer.innerHTML = '<p>No active polls found.</p>';
            } else {
                renderPolls(polls, activePollsContainer);
            }
        } catch (error) {
            activePollsContainer.innerHTML = `<p class="error-message">Error loading polls: ${error.message}</p>`;
        } finally {
            activePollsLoading.style.display = 'none';
        }
    }
    
    // Load closed polls
    async function loadClosedPolls() {
        try {
            closedPollsLoading.style.display = 'block';
            closedPollsContainer.innerHTML = '';
            
            const polls = await api.getInactivePolls();
            
            if (polls.length === 0) {
                closedPollsContainer.innerHTML = '<p>No closed polls found.</p>';
            } else {
                renderPolls(polls, closedPollsContainer);
            }
        } catch (error) {
            closedPollsContainer.innerHTML = `<p class="error-message">Error loading polls: ${error.message}</p>`;
        } finally {
            closedPollsLoading.style.display = 'none';
        }
    }
    
    // Render polls to container
    function renderPolls(polls, container) {
        polls.forEach(poll => {
            const card = document.createElement('div');
            card.className = 'card poll-card';
            card.dataset.id = poll.id;
            
            const expiryText = poll.expires_at ? formatDate(new Date(poll.expires_at)) : 'No expiration';
            
            card.innerHTML = `
                <div class="card-body">
                    <h3 class="poll-title">${escapeHtml(poll.title)}</h3>
                    <p class="poll-description">${escapeHtml(poll.description || 'No description')}</p>
                    <div class="poll-metadata">
                        <p>Created: ${formatDate(new Date(poll.created_at))}</p>
                        <p>Expires: ${expiryText}</p>
                    </div>
                </div>
            `;
            
            card.addEventListener('click', () => openPollModal(poll.id));
            container.appendChild(card);
        });
    }
    
    // Open poll modal
    async function openPollModal(pollId) {
        try {
            resetModal();
            pollModal.style.display = 'block';
            
            // Load poll details
            const pollDetails = await api.getPoll(pollId);
            const poll = pollDetails.poll;
            const options = pollDetails.options;
            const userVote = pollDetails.user_vote;
            
            // Set modal content
            modalPollTitle.textContent = poll.title;
            modalPollDescription.textContent = poll.description || 'No description';
            modalPollCreated.textContent = formatDate(new Date(poll.created_at));
            
            // Set expiry if exists
            if (poll.expires_at) {
                modalPollExpiryContainer.style.display = 'block';
                modalPollExpiry.textContent = formatDate(new Date(poll.expires_at));
            } else {
                modalPollExpiryContainer.style.display = 'none';
            }
            
            // Fetch creator's username
            try {
                const users = await api.listUsers();
                const creator = users.find(user => user.id === poll.created_by);
                modalPollCreator.textContent = creator ? creator.username : 'Unknown';
            } catch (error) {
                modalPollCreator.textContent = 'Unknown';
            }
            
            // Render voting options
            renderVotingOptions(options, userVote, poll.is_active);
            
            // Load poll results
            loadPollResults(pollId);
            
            // Store poll ID in form for voting
            voteForm.dataset.pollId = pollId;
            
        } catch (error) {
            console.error('Error loading poll details:', error);
            pollModal.style.display = 'none';
            alert('Error loading poll details. Please try again.');
        }
    }
    
    // Render voting options
    function renderVotingOptions(options, userVote, isActive) {
        modalPollOptions.innerHTML = '';
        
        options.forEach(option => {
            const optionElement = document.createElement('div');
            
            let optionText = option.text;
            if (option.datetime_option) {
                optionText += ` - ${formatDate(new Date(option.datetime_option))}`;
            }
            
            const isChecked = userVote === option.id ? 'checked' : '';
            const disabled = !isActive ? 'disabled' : '';
            
            optionElement.innerHTML = `
                <input type="radio" name="option" value="${option.id}" id="option-${option.id}" ${isChecked} ${disabled}>
                <label for="option-${option.id}">${escapeHtml(optionText)}</label>
            `;
            
            modalPollOptions.appendChild(optionElement);
        });
        
        // Show/hide voting section based on poll status
        if (!isActive) {
            document.getElementById('modal-poll-voting-section').style.display = 'none';
        } else {
            document.getElementById('modal-poll-voting-section').style.display = 'block';
        }
    }
    
    // Load poll results
    async function loadPollResults(pollId) {
        try {
            resultsLoading.style.display = 'block';
            pollResultsContainer.innerHTML = '';
            
            const results = await api.getPollResults(pollId);
            
            pollResultsContainer.innerHTML = `
                <p>Total votes: ${results.total_votes}</p>
            `;
            
            results.options.forEach(option => {
                const resultElement = document.createElement('div');
                resultElement.className = 'poll-result';
                
                let optionText = option.text;
                if (option.datetime_option) {
                    optionText += ` - ${formatDate(new Date(option.datetime_option))}`;
                }
                
                const isUserVote = results.user_vote === option.id;
                const fillClass = isUserVote ? 'poll-result-fill your-vote' : 'poll-result-fill';
                
                resultElement.innerHTML = `
                    <div class="poll-result-text">
                        <span>${escapeHtml(optionText)}${isUserVote ? ' (Your vote)' : ''}</span>
                        <span>${option.vote_count} votes</span>
                    </div>
                    <div class="poll-result-bar">
                        <div class="${fillClass}" style="width: ${option.percentage}%">
                            ${option.percentage.toFixed(1)}%
                        </div>
                    </div>
                `;
                
                pollResultsContainer.appendChild(resultElement);
            });
            
        } catch (error) {
            pollResultsContainer.innerHTML = `<p class="error-message">Error loading results: ${error.message}</p>`;
        } finally {
            resultsLoading.style.display = 'none';
        }
    }
    
    // Vote form submission
    voteForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        voteError.textContent = '';
        voteSuccess.textContent = '';
        
        const selectedOption = document.querySelector('input[name="option"]:checked');
        if (!selectedOption) {
            voteError.textContent = 'Please select an option';
            return;
        }
        
        const pollId = voteForm.dataset.pollId;
        const optionId = parseInt(selectedOption.value);
        
        try {
            await api.votePoll(pollId, optionId);
            voteSuccess.textContent = 'Vote recorded successfully!';
            
            // Reload poll results
            loadPollResults(pollId);
        } catch (error) {
            voteError.textContent = error.data?.error || 'Failed to record vote. Please try again.';
        }
    });
    
    // Close modal
    modalClose.addEventListener('click', () => {
        pollModal.style.display = 'none';
    });
    
    // Close modal when clicking outside
    window.addEventListener('click', (e) => {
        if (e.target === pollModal) {
            pollModal.style.display = 'none';
        }
    });
    
    // Reset modal state
    function resetModal() {
        modalPollTitle.textContent = '';
        modalPollDescription.textContent = '';
        modalPollCreator.textContent = '';
        modalPollCreated.textContent = '';
        modalPollExpiry.textContent = '';
        modalPollOptions.innerHTML = '';
        pollResultsContainer.innerHTML = '';
        voteError.textContent = '';
        voteSuccess.textContent = '';
    }
    
    // Add poll option button
    addOptionBtn.addEventListener('click', () => {
        const optionsCount = document.querySelectorAll('.poll-option').length;
        const newIndex = optionsCount + 1;
        
        const optionElement = document.createElement('div');
        optionElement.className = 'poll-option';
        optionElement.innerHTML = `
            <input type="text" name="option-text-${newIndex}" placeholder="Option text" required>
            <div class="option-date-container">
                <input type="checkbox" class="date-option-toggle" id="date-toggle-${newIndex}">
                <label for="date-toggle-${newIndex}" class="date-toggle-label">Include date/time</label>
                <input type="datetime-local" name="option-date-${newIndex}" class="date-input" style="display: none;">
            </div>
        `;
        
        pollOptionsContainer.appendChild(optionElement);
        
        // Add event listener for the new date toggle
        const newToggle = document.getElementById(`date-toggle-${newIndex}`);
        const newDateInput = optionElement.querySelector('.date-input');
        
        newToggle.addEventListener('change', () => {
            newDateInput.style.display = newToggle.checked ? 'block' : 'none';
            if (!newToggle.checked) {
                newDateInput.value = '';
            }
        });
    });
    
    // Set up event listeners for existing date toggles
    document.querySelectorAll('.date-option-toggle').forEach(toggle => {
        toggle.addEventListener('change', function() {
            const dateInput = this.parentElement.querySelector('.date-input');
            dateInput.style.display = this.checked ? 'block' : 'none';
            if (!this.checked) {
                dateInput.value = '';
            }
        });
    });
    
    // Create poll form submission
    createPollForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        createPollError.textContent = '';
        createPollSuccess.textContent = '';
        
        try {
            // Gather poll data
            const options = [];
            const optionElements = document.querySelectorAll('.poll-option');
            
            optionElements.forEach((el, index) => {
                const textInput = el.querySelector(`input[name^="option-text"]`);
                const dateToggle = el.querySelector(`input[type="checkbox"]`);
                const dateInput = el.querySelector(`input[name^="option-date"]`);
                
                if (textInput.value.trim()) {
                    const option = {
                        text: textInput.value.trim()
                    };
                    
                    if (dateToggle.checked && dateInput.value) {
                        option.datetime_option = new Date(dateInput.value).toISOString();
                    }
                    
                    options.push(option);
                }
            });
            
            if (options.length < 2) {
                createPollError.textContent = 'Please add at least 2 options';
                return;
            }
            
            const pollData = {
                title: pollTitle.value.trim(),
                description: pollDescription.value.trim() || null,
                options: options
            };
            
            if (pollExpiry.value) {
                pollData.expires_at = new Date(pollExpiry.value).toISOString();
            }
            
            // Create the poll
            await api.createPoll(pollData);
            
            // Clear form
            createPollForm.reset();
            
            // Show only 2 default options
            pollOptionsContainer.innerHTML = `
                <div class="poll-option">
                    <input type="text" name="option-text-1" placeholder="Option text" required>
                    <div class="option-date-container">
                        <input type="checkbox" class="date-option-toggle" id="date-toggle-1">
                        <label for="date-toggle-1" class="date-toggle-label">Include date/time</label>
                        <input type="datetime-local" name="option-date-1" class="date-input" style="display: none;">
                    </div>
                </div>
                <div class="poll-option">
                    <input type="text" name="option-text-2" placeholder="Option text" required>
                    <div class="option-date-container">
                        <input type="checkbox" class="date-option-toggle" id="date-toggle-2">
                        <label for="date-toggle-2" class="date-toggle-label">Include date/time</label>
                        <input type="datetime-local" name="option-date-2" class="date-input" style="display: none;">
                    </div>
                </div>
            `;
            
            // Re-add event listeners for date toggles
            document.querySelectorAll('.date-option-toggle').forEach(toggle => {
                toggle.addEventListener('change', function() {
                    const dateInput = this.parentElement.querySelector('.date-input');
                    dateInput.style.display = this.checked ? 'block' : 'none';
                    if (!this.checked) {
                        dateInput.value = '';
                    }
                });
            });
            
            // Show success message
            createPollSuccess.textContent = 'Poll created successfully!';
            
            // Switch to active polls tab and reload
            setTimeout(() => {
                showTab(activePollsTab, activePollsContent);
                loadActivePolls();
                createPollSuccess.textContent = '';
            }, 2000);
            
        } catch (error) {
            createPollError.textContent = error.data?.error || 'Failed to create poll. Please try again.';
        }
    });
    
    // Helper functions
    function formatDate(date) {
        return new Intl.DateTimeFormat('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: 'numeric',
            minute: 'numeric'
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