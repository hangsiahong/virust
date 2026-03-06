// State
let todos = [];

// DOM elements
const todoForm = document.getElementById('todoForm');
const titleInput = document.getElementById('titleInput');
const descriptionInput = document.getElementById('descriptionInput');
const todosList = document.getElementById('todosList');

// Load todos from server
async function loadTodos() {
    try {
        const response = await fetch('/api/todos');
        todos = await response.json();
        renderTodos();
    } catch (err) {
        console.error('Failed to load todos:', err);
        todosList.innerHTML = '';
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error';
        errorDiv.textContent = 'Failed to load todos. Please refresh the page.';
        todosList.appendChild(errorDiv);
    }
}

// Render todos to the DOM
function renderTodos() {
    todosList.innerHTML = '';

    if (todos.length === 0) {
        const emptyDiv = document.createElement('div');
        emptyDiv.className = 'empty-state';
        emptyDiv.textContent = 'No todos yet. Add one above!';
        todosList.appendChild(emptyDiv);
        return;
    }

    // Sort: incomplete first, then by creation date
    const sortedTodos = [...todos].sort((a, b) => {
        if (a.completed !== b.completed) {
            return a.completed ? 1 : -1;
        }
        return a.created_at - b.created_at;
    });

    sortedTodos.forEach(todo => {
        const todoDiv = document.createElement('div');
        todoDiv.className = `todo-item ${todo.completed ? 'completed' : ''}`;
        todoDiv.dataset.id = todo.id;

        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.className = 'todo-checkbox';
        checkbox.checked = todo.completed;
        checkbox.addEventListener('change', () => toggleTodo(todo.id, checkbox.checked));

        const contentDiv = document.createElement('div');
        contentDiv.className = 'todo-content';

        const titleDiv = document.createElement('div');
        titleDiv.className = 'todo-title';
        titleDiv.textContent = todo.title;

        contentDiv.appendChild(titleDiv);

        if (todo.description) {
            const descDiv = document.createElement('div');
            descDiv.className = 'todo-description';
            descDiv.textContent = todo.description;
            contentDiv.appendChild(descDiv);
        }

        const actionsDiv = document.createElement('div');
        actionsDiv.className = 'todo-actions';

        const deleteButton = document.createElement('button');
        deleteButton.className = 'delete-button';
        deleteButton.textContent = 'Delete';
        deleteButton.addEventListener('click', () => deleteTodo(todo.id));

        actionsDiv.appendChild(deleteButton);

        todoDiv.appendChild(checkbox);
        todoDiv.appendChild(contentDiv);
        todoDiv.appendChild(actionsDiv);

        todosList.appendChild(todoDiv);
    });
}

// Add a new todo
async function addTodo(title, description) {
    try {
        const response = await fetch('/api/todos', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                title: title,
                description: description || null,
            }),
        });

        if (response.ok) {
            const newTodo = await response.json();
            todos.push(newTodo);
            renderTodos();
            titleInput.value = '';
            descriptionInput.value = '';
            titleInput.focus();
        } else {
            alert('Failed to add todo');
        }
    } catch (err) {
        console.error('Failed to add todo:', err);
        alert('Failed to add todo');
    }
}

// Toggle todo completion
async function toggleTodo(id, completed) {
    try {
        const todo = todos.find(t => t.id === id);
        if (!todo) return;

        const response = await fetch(`/api/todos/${id}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                completed: completed,
            }),
        });

        if (response.ok) {
            const updated = await response.json();
            Object.assign(todo, updated);
            renderTodos();
        } else {
            alert('Failed to update todo');
            renderTodos(); // Revert checkbox
        }
    } catch (err) {
        console.error('Failed to update todo:', err);
        alert('Failed to update todo');
        renderTodos(); // Revert checkbox
    }
}

// Delete a todo
async function deleteTodo(id) {
    if (!confirm('Are you sure you want to delete this todo?')) {
        return;
    }

    try {
        const response = await fetch(`/api/todos/${id}`, {
            method: 'DELETE',
        });

        if (response.ok) {
            todos = todos.filter(t => t.id !== id);
            renderTodos();
        } else {
            alert('Failed to delete todo');
        }
    } catch (err) {
        console.error('Failed to delete todo:', err);
        alert('Failed to delete todo');
    }
}

// Form submission
todoForm.addEventListener('submit', (e) => {
    e.preventDefault();
    const title = titleInput.value.trim();
    const description = descriptionInput.value.trim();

    if (title) {
        addTodo(title, description);
    }
});

// Initialize
loadTodos();
