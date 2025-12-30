import { invoke } from '@tauri-apps/api/core';

// 应用状态管理
const AppState = {
    connected: false,
    serverUrl: '',
    messages: [],
    loading: false,
    error: null
};

// 更新UI状态的辅助函数
function updateUIState(updates) {
    Object.assign(AppState, updates);
    render();
}

function render() {
    // 更新连接状态指示器
    const statusIndicator = document.getElementById('statusIndicator');
    const statusText = document.getElementById('statusText');
    if (statusIndicator && statusText) {
        statusIndicator.className = AppState.connected ? 'status connected' : 'status disconnected';
        statusText.textContent = AppState.connected ? '已连接' : '未连接';
    }

    // 更新按钮状态
    const connectBtn = document.getElementById('connectBtn');
    const disconnectBtn = document.getElementById('disconnectBtn');
    if (connectBtn) {
        connectBtn.disabled = AppState.loading;
        connectBtn.textContent = AppState.loading ? '连接中...' : '连接';
    }
    if (disconnectBtn) {
        disconnectBtn.style.display = AppState.connected ? 'inline-block' : 'none';
    }

    // 更新错误提示
    const errorDiv = document.getElementById('errorMessage');
    if (errorDiv) {
        if (AppState.error) {
            errorDiv.textContent = AppState.error;
            errorDiv.style.display = 'block';
        } else {
            errorDiv.style.display = 'none';
        }
    }

    // 更新消息列表
    renderMessages();
}

function renderMessages() {
    const msgDiv = document.getElementById('messages');
    if (!msgDiv) return;

    if (AppState.messages.length === 0) {
        msgDiv.innerHTML = '<p class="empty-state">暂无消息</p>';
        return;
    }

    msgDiv.innerHTML = `
        <div class="messages-header">
            <h2>消息 (${AppState.messages.length})</h2>
            <button id="refreshBtn" onclick="refreshMessages()">刷新</button>
        </div>
        ${AppState.messages.map(m => `
            <div class="message-card priority-${m.priority}" data-id="${m.id}">
                <div class="message-header">
                    <span class="message-title">${escapeHtml(m.title || '无标题')}</span>
                    <span class="message-time">${formatTime(m.timestamp)}</span>
                </div>
                <div class="message-body">${escapeHtml(m.message)}</div>
                <div class="message-footer">
                    <span class="priority-badge priority-${m.priority}">优先级: ${m.priority}</span>
                    <button class="delete-btn" onclick="deleteMessage(${m.id})">删除</button>
                </div>
            </div>
        `).join('')}
    `;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatTime(timestamp) {
    try {
        const date = new Date(timestamp);
        return date.toLocaleString('zh-CN');
    } catch {
        return timestamp;
    }
}

// 连接 Gotify 服务器
document.getElementById('connectBtn').addEventListener('click', async () => {
    const url = document.getElementById('serverUrl').value.trim();
    const token = document.getElementById('token').value.trim();

    if (!url) {
        updateUIState({ error: '请输入服务器 URL' });
        return;
    }
    if (!token) {
        updateUIState({ error: '请输入 Token' });
        return;
    }

    // 验证 URL 格式
    try {
        new URL(url);
    } catch {
        updateUIState({ error: '无效的 URL 格式' });
        return;
    }

    updateUIState({ loading: true, error: null });

    try {
        const result = await invoke('connect_to_gotify', { 
            req: { server_url: url, token } 
        });

        if (result.success) {
            AppState.connected = true;
            AppState.serverUrl = url;
            updateUIState({ loading: false });
            await refreshMessages();
        } else {
            updateUIState({ 
                loading: false, 
                error: result.error || '连接失败' 
            });
        }
    } catch (e) {
        updateUIState({ loading: false, error: `连接错误: ${e}` });
    }
});

// 断开连接
document.getElementById('disconnectBtn')?.addEventListener('click', async () => {
    try {
        await invoke('disconnect_gotify');
        AppState.connected = false;
        AppState.messages = [];
        updateUIState({ error: null });
    } catch (e) {
        updateUIState({ error: `断开连接错误: ${e}` });
    }
});

// 刷新消息列表
async function refreshMessages() {
    if (!AppState.connected) return;

    updateUIState({ loading: true, error: null });

    try {
        const result = await invoke('fetch_messages', { since: null });
        if (result.success) {
            AppState.messages = result.data || [];
            updateUIState({ loading: false });
        } else {
            updateUIState({ 
                loading: false, 
                error: result.error || '获取消息失败' 
            });
        }
    } catch (e) {
        updateUIState({ loading: false, error: `获取消息错误: ${e}` });
    }
}

// 删除消息
async function deleteMessage(messageId) {
    if (!confirm('确定要删除这条消息吗？')) return;

    try {
        const result = await invoke('delete_message', { messageId });
        if (result.success) {
            AppState.messages = AppState.messages.filter(m => m.id !== messageId);
            updateUIState({ error: null });
        } else {
            updateUIState({ error: result.error || '删除失败' });
        }
    } catch (e) {
        updateUIState({ error: `删除消息错误: ${e}` });
    }
}

// 定期刷新消息（每30秒）
setInterval(() => {
    if (AppState.connected) {
        refreshMessages();
    }
}, 30000);

// 初始渲染
document.addEventListener('DOMContentLoaded', () => {
    render();
});