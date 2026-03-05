// WebSocket connection
let ws = null;
let username = null;
let reconnectInterval = null;

// DOM elements
const connectionStatus = document.getElementById('connectionStatus');
const chatMessages = document.getElementById('chatMessages');
const usernameInput = document.getElementById('usernameInput');
const messageInput = document.getElementById('messageInput');
const messageInputGroup = document.getElementById('messageInputGroup');
const sendButton = document.getElementById('sendButton');

// Initialize connection
function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws/api/route`;

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        updateConnectionStatus(true);
        console.log('WebSocket connected');

        // Load message history
        loadHistory();
    };

    ws.onmessage = (event) => {
        try {
            const response = JSON.parse(event.data);
            console.log('Received:', response);

            if (response.ok) {
                // Reload history to show new message
                loadHistory();
            }
        } catch (err) {
            console.error('Failed to parse message:', err);
        }
    };

    ws.onclose = () => {
        updateConnectionStatus(false);
        console.log('WebSocket disconnected');

        // Attempt to reconnect
        if (!reconnectInterval) {
            reconnectInterval = setInterval(() => {
                console.log('Attempting to reconnect...');
                connect();
            }, 3000);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
}

function updateConnectionStatus(connected) {
    const dot = connectionStatus.querySelector('.status-dot');
    const text = connectionStatus.querySelector('.status-text');

    if (connected) {
        dot.classList.remove('disconnected');
        dot.classList.add('connected');
        text.textContent = 'Connected';

        if (reconnectInterval) {
            clearInterval(reconnectInterval);
            reconnectInterval = null;
        }
    } else {
        dot.classList.remove('connected');
        dot.classList.add('disconnected');
        text.textContent = 'Disconnected';
    }
}

async function loadHistory() {
    try {
        const response = await fetch('/api/history');
        const history = await response.json();

        // Clear chat messages
        chatMessages.innerHTML = '';

        if (history.length === 0) {
            const welcomeDiv = document.createElement('div');
            welcomeDiv.className = 'welcome-message';
            welcomeDiv.textContent = 'No messages yet. Start the conversation!';
            chatMessages.appendChild(welcomeDiv);
            return;
        }

        // Display messages
        history.forEach(entry => {
            addMessageToChat(entry);
        });

        // Scroll to bottom
        chatMessages.scrollTop = chatMessages.scrollHeight;
    } catch (err) {
        console.error('Failed to load history:', err);
    }
}

function addMessageToChat(entry) {
    const messageDiv = document.createElement('div');
    messageDiv.className = 'chat-message';

    const headerDiv = document.createElement('div');
    headerDiv.className = 'message-header';

    const usernameSpan = document.createElement('span');
    usernameSpan.className = 'message-username';
    usernameSpan.textContent = entry.username;

    const timeSpan = document.createElement('span');
    timeSpan.className = 'message-time';
    const timestamp = new Date(entry.timestamp * 1000).toLocaleTimeString();
    timeSpan.textContent = timestamp;

    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';
    contentDiv.textContent = entry.message;

    headerDiv.appendChild(usernameSpan);
    headerDiv.appendChild(timeSpan);
    messageDiv.appendChild(headerDiv);
    messageDiv.appendChild(contentDiv);

    chatMessages.appendChild(messageDiv);
}

function sendMessage() {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        alert('Not connected. Please wait for reconnection.');
        return;
    }

    const message = messageInput.value.trim();
    if (!message) {
        return;
    }

    const payload = {
        username: username,
        message: message
    };

    ws.send(JSON.stringify(payload));
    messageInput.value = '';
}

// Event listeners
usernameInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        username = usernameInput.value.trim();
        if (username) {
            usernameInput.style.display = 'none';
            messageInputGroup.style.display = 'flex';
            messageInput.focus();
        }
    }
});

messageInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        sendMessage();
    }
});

sendButton.addEventListener('click', sendMessage);

// Initialize
connect();
