// 使用全局 Tauri API
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// 应用状态管理
const AppState = {
    connected: false,
    serverUrl: '',
    messages: [],
    loading: false,
    error: null,
    loadingMore: false,
    hasMoreMessages: true,
    configs: [],
    currentConfigId: null,
    settings: {
        enable_autostart: false,
        minimize_to_tray: false,
        silent_start: false,
        enable_notifications: false
    }
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

    // 更新配置名称和服务器地址显示
    const configDisplay = document.getElementById('configDisplay');
    if (configDisplay) {
        if (AppState.connected && AppState.configName) {
            configDisplay.textContent = `${AppState.configName} (${AppState.serverUrl})`;
            configDisplay.style.display = 'inline-block';
        } else {
            configDisplay.style.display = 'none';
        }
    }

    // 更新按钮状态
    const connectBtn = document.getElementById('connectBtn');
    const disconnectBtn = document.getElementById('disconnectBtn');
    if (connectBtn) {
        connectBtn.disabled = AppState.loading;
        connectBtn.textContent = AppState.loading ? '连接中...' : '连接';
        connectBtn.style.display = AppState.connected ? 'none' : 'inline-block';
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
            <h2>消息</h2>
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
        ${AppState.loadingMore ? '<div class="loading-more">加载中...</div>' : ''}
        ${AppState.hasMoreMessages ? '<button class="load-more-btn" onclick="loadMoreMessages()">加载更多</button>' : ''}
    `;
}

// 加载更多消息
async function loadMoreMessages() {
    if (!AppState.connected || AppState.loadingMore || !AppState.hasMoreMessages) return;

    console.log('loadMoreMessages called, current messages:', AppState.messages.length);
    AppState.loadingMore = true;
    renderMessages();

    try {
        // 使用 offset 和 limit 来加载更多消息
        const offset = AppState.messages.length;
        
        console.log('Fetching with offset:', offset, 'limit:', 10);
        const result = await invoke('fetch_messages', { 
            since: null,
            limit: 10,
            offset: offset
        });
        console.log('Fetch result:', result);
        if (result.success) {
            const newMessages = result.data || [];
            console.log('New messages received:', newMessages.length);
            if (newMessages.length > 0) {
                AppState.messages = [...AppState.messages, ...newMessages];
                AppState.hasMoreMessages = newMessages.length >= 10;
                console.log('Total messages:', AppState.messages.length, 'hasMore:', AppState.hasMoreMessages);
            } else {
                AppState.hasMoreMessages = false;
            }
        } else {
            console.error('Fetch failed:', result.error);
        }
    } catch (e) {
        console.error('加载更多消息错误:', e);
    } finally {
        AppState.loadingMore = false;
        renderMessages();
    }
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
// 事件监听器会在 DOMContentLoaded 中设置

// 断开连接
// 事件监听器会在 DOMContentLoaded 中设置

// 刷新消息列表
async function refreshMessages() {
    if (!AppState.connected) return;

    updateUIState({ loading: true, error: null });

    try {
        console.log('Fetching messages...');
        const result = await invoke('fetch_messages', { 
            since: null,
            limit: 10,
            offset: 0
        });
        console.log('Fetch result:', result);
        if (result.success) {
            const allMessages = result.data || [];
            AppState.messages = allMessages;
            AppState.hasMoreMessages = allMessages.length >= 10;
            console.log('Messages loaded:', AppState.messages.length, 'hasMore:', AppState.hasMoreMessages);
            updateUIState({ loading: false });
        } else {
            console.error('Fetch failed:', result.error);
            updateUIState({
                loading: false,
                error: result.error || '获取消息失败'
            });
        }
    } catch (e) {
        console.error('Fetch error:', e);
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

// 保存配置
document.getElementById('saveConfigBtn')?.addEventListener('click', async () => {
    const url = document.getElementById('serverUrl').value.trim();
    const token = document.getElementById('token').value.trim();
    const name = document.getElementById('configName').value.trim() || url;
    const isDefault = document.getElementById('isDefaultConfig').checked;

    if (!url || !token) {
        updateUIState({ error: '请输入服务器 URL 和 Token' });
        return;
    }

    try {
        new URL(url);
    } catch {
        updateUIState({ error: '无效的 URL 格式' });
        return;
    }

    updateUIState({ loading: true, error: null });

    try {
        const result = await invoke('save_config', {
            config: {
                id: generateId(),
                name,
                server_url: url,
                token,
                is_default: isDefault
            }
        });

        if (result.success) {
            // 如果设置为默认，更新其他配置
            if (isDefault) {
                await invoke('set_default_config', { id: result.data?.id || generateId() });
            }

            updateUIState({ loading: false, error: null });
            await loadConfigs();
            document.getElementById('configName').value = '';
            document.getElementById('isDefaultConfig').checked = false;
        } else {
            updateUIState({
                loading: false,
                error: result.error || '保存配置失败'
            });
        }
    } catch (e) {
        updateUIState({ loading: false, error: `保存配置错误: ${e}` });
    }
});

// 加载配置列表
async function loadConfigs() {
    try {
        console.log('Loading configs...');
        const result = await invoke('get_configs');
        console.log('Got configs result:', result);
        if (result.success) {
            AppState.configs = result.data || [];
            console.log('AppState.configs:', AppState.configs);
            renderConfigs();
        }
    } catch (e) {
        console.error('加载配置错误:', e);
    }
}

// 渲染配置列表
function renderConfigs() {
    const configList = document.getElementById('configList');

    if (!configList) return;

    console.log('renderConfigs called, currentConfigId:', AppState.currentConfigId, 'configs:', AppState.configs);

    if (AppState.configs.length === 0) {
        configList.innerHTML = `
            <div class="empty-state">
                <i class="ri-server-line" style="font-size: 48px; color: #ccc; margin-bottom: 10px;"></i>
                <p>暂无配置</p>
                <p style="font-size: 0.9rem; color: #999;">点击右下角的按钮添加新配置</p>
            </div>
        `;
        return;
    }

    // 按最后使用时间排序
    const sortedConfigs = [...AppState.configs].sort((a, b) => {
        if (!a.last_used) return 1;
        if (!b.last_used) return -1;
        return b.last_used.localeCompare(a.last_used);
    });

    configList.innerHTML = `
        <div class="config-grid">
            ${sortedConfigs.map(config => `
                <div class="config-card ${config.last_used ? 'default' : ''} ${AppState.currentConfigId === config.id ? 'connected' : ''}">
                    <div class="config-header">
                        <div class="config-name">
                            ${escapeHtml(config.name)}
                            ${AppState.currentConfigId === config.id ? '<span class="connected-badge"><i class="ri-checkbox-circle-fill"></i> 已连接</span>' : ''}
                        </div>
                        ${config.last_used ? `<div class="config-last-used">最后使用: ${formatLastUsed(config.last_used)}</div>` : ''}
                    </div>
                    <div class="config-url">${escapeHtml(config.server_url)}</div>
                    <div class="config-actions">
                        ${AppState.currentConfigId === config.id ? `
                            <button class="config-action-btn disabled" disabled>
                                <i class="ri-link"></i> 已连接
                            </button>
                        ` : `
                            <button class="config-action-btn primary" onclick="useConfig('${config.id}')">
                                <i class="ri-link"></i> 连接
                            </button>
                        `}
                        <button class="config-action-btn secondary" onclick="editConfig('${config.id}')">
                            <i class="ri-edit-line"></i> 编辑
                        </button>
                        <button class="config-action-btn danger" onclick="deleteConfig('${config.id}')">
                            <i class="ri-delete-bin-line"></i> 删除
                        </button>
                    </div>
                </div>
            `).join('')}
        </div>
    `;
}

function formatLastUsed(timestamp) {
    try {
        const date = new Date(timestamp);
        const now = new Date();
        const diff = now - date;

        if (diff < 60000) return '刚刚';
        if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
        if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
        return date.toLocaleDateString('zh-CN');
    } catch {
        return timestamp;
    }
}

// 使用已保存的配置
async function useConfig(id) {
    const config = AppState.configs.find(c => c.id === id);
    if (!config) return;

    // 切换到消息页面
    switchPage('messages');

    // 直接连接，不依赖表单元素
    await connectToServer(config.server_url, config.token, config.name, id);

    // 更新最后使用时间
    await invoke('set_default_config', { id });
    await loadConfigs();
}

// 删除配置
async function deleteConfig(id) {
    if (!confirm('确定要删除这个配置吗？')) return;

    try {
        const result = await invoke('delete_config', { id });
        if (result.success) {
            AppState.configs = AppState.configs.filter(c => c.id !== id);
            renderConfigs();
        } else {
            updateUIState({ error: result.error || '删除配置失败' });
        }
    } catch (e) {
        updateUIState({ error: `删除配置错误: ${e}` });
    }
}

// 暴露函数到全局作用域
window.useConfig = useConfig;
window.deleteConfig = deleteConfig;
window.setDefaultConfig = setDefaultConfig;
window.deleteMessage = deleteMessage;
window.refreshMessages = refreshMessages;
window.switchPage = switchPage;
window.toggleSidebar = toggleSidebar;

function generateId() {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// 切换页面
function switchPage(pageName) {
    AppState.currentPage = pageName;

    // 更新导航项状态
    document.querySelectorAll('.nav-item').forEach(item => {
        if (item.dataset.page === pageName) {
            item.classList.add('active');
        } else {
            item.classList.remove('active');
        }
    });

    // 更新页面显示
    document.querySelectorAll('.page').forEach(page => {
        if (page.id === `page-${pageName}`) {
            page.style.display = 'block';
        } else {
            page.style.display = 'none';
        }
    });

    // 如果切换到服务器页面，重新渲染配置列表
    if (pageName === 'servers') {
        renderConfigs();
    }
}

// 切换侧边栏
function toggleSidebar() {
    const sidebar = document.getElementById('sidebar');
    const mainContent = document.querySelector('.main-content');
    
    sidebar.classList.toggle('collapsed');
    
    // 动态调整主内容的左边距
    if (sidebar.classList.contains('collapsed')) {
        mainContent.style.marginLeft = '50px';
    } else {
        mainContent.style.marginLeft = '200px';
    }
    
    AppState.sidebarCollapsed = sidebar.classList.contains('collapsed');

    // 保存侧边栏状态
    try {
        localStorage.setItem('sidebarCollapsed', AppState.sidebarCollapsed);
    } catch (e) {
        console.error('Failed to save sidebar state:', e);
    }
}

// 添加悬浮提示
function setupTooltips() {
    const navItems = document.querySelectorAll('.nav-item[data-title]');
    console.log('Found nav items:', navItems.length);
    const delay = 600; // 600ms 延迟
    let timeoutId = null;

    navItems.forEach(item => {
        const tooltip = document.createElement('div');
        tooltip.className = 'nav-tooltip';
        tooltip.textContent = item.getAttribute('data-title');
        item.appendChild(tooltip);

        item.addEventListener('mouseenter', (e) => {
            console.log('Mouse enter on nav item');
            if (document.getElementById('sidebar').classList.contains('collapsed')) {
                // 清除之前的定时器
                if (timeoutId) {
                    clearTimeout(timeoutId);
                }
                // 设置新的定时器
                timeoutId = setTimeout(() => {
                    const rect = item.getBoundingClientRect();
                    tooltip.style.top = rect.top + 'px';
                    tooltip.style.display = 'block';
                    console.log('Showing tooltip at:', rect.top);
                }, delay);
            }
        });

        item.addEventListener('mouseleave', () => {
            console.log('Mouse leave from nav item');
            // 清除定时器
            if (timeoutId) {
                clearTimeout(timeoutId);
                timeoutId = null;
            }
            tooltip.style.display = 'none';
        });
    });
}

// 设置默认配置
async function setDefaultConfig(id) {
    try {
        const result = await invoke('set_default_config', { id });
        if (result.success) {
            await loadConfigs();
        } else {
            updateUIState({ error: result.error || '设置默认配置失败' });
        }
    } catch (e) {
        updateUIState({ error: `设置默认配置错误: ${e}` });
    }
}

// 显示新增配置模态框
function showAddConfigModal() {
    document.getElementById('modalTitle').textContent = '新增配置';
    document.getElementById('modalServerUrl').value = '';
    document.getElementById('modalToken').value = '';
    document.getElementById('modalConfigName').value = '';
    document.getElementById('configModal').classList.add('show');
}

// 显示编辑配置模态框
function editConfig(id) {
    const config = AppState.configs.find(c => c.id === id);
    if (!config) return;

    document.getElementById('modalTitle').textContent = '编辑配置';
    document.getElementById('modalServerUrl').value = config.server_url;
    document.getElementById('modalToken').value = config.token;
    document.getElementById('modalConfigName').value = config.name;
    document.getElementById('configModal').classList.add('show');
    document.getElementById('configModal').dataset.editId = id;
}

// 关闭配置模态框
function closeConfigModal() {
    document.getElementById('configModal').classList.remove('show');
    delete document.getElementById('configModal').dataset.editId;
}

// 从模态框保存配置
async function saveConfigFromModal() {
    const serverUrl = document.getElementById('modalServerUrl').value.trim();
    const token = document.getElementById('modalToken').value.trim();
    const configName = document.getElementById('modalConfigName').value.trim();
    const modal = document.getElementById('configModal');
    const editId = modal.dataset.editId;

    if (!serverUrl) {
        alert('请输入服务器 URL');
        return;
    }
    if (!token) {
        alert('请输入 Token');
        return;
    }
    if (!configName) {
        alert('请输入配置名称');
        return;
    }

    // 验证 URL 格式
    try {
        new URL(serverUrl);
    } catch {
        alert('无效的 URL 格式');
        return;
    }

    try {
        if (editId) {
            // 编辑现有配置
            console.log('Updating config:', { id: editId, name: configName, serverUrl: serverUrl });
            const result = await invoke('update_config', {
                id: editId,
                name: configName,
                serverUrl: serverUrl,
                token
            });
            if (result.success) {
                await loadConfigs();
                closeConfigModal();
            } else {
                alert(result.error || '更新配置失败');
            }
        } else {
            // 新增配置
            console.log('Saving config:', { name: configName, serverUrl: serverUrl });
            const result = await invoke('save_config', {
                name: configName,
                serverUrl: serverUrl,
                token
            });
            if (result.success) {
                await loadConfigs();
                closeConfigModal();
            } else {
                alert(result.error || '保存配置失败');
            }
        }
    } catch (e) {
        console.error('Save config error:', e);
        alert(`保存配置错误: ${e}`);
    }
}

// 连接到服务器
async function connectToServer(serverUrl, token, configName = null, configId = null) {
    if (!serverUrl) {
        updateUIState({ error: '请输入服务器 URL' });
        return;
    }
    if (!token) {
        updateUIState({ error: '请输入 Token' });
        return;
    }

    // 验证 URL 格式
    try {
        new URL(serverUrl);
    } catch {
        updateUIState({ error: '无效的 URL 格式' });
        return;
    }

    console.log('connectToServer called with configId:', configId);
    updateUIState({ loading: true, error: null });

    try {
        const result = await invoke('connect_to_gotify', { 
            req: { server_url: serverUrl, token } 
        });

        if (result.success) {
            AppState.connected = true;
            AppState.serverUrl = serverUrl;
            if (configName) {
                AppState.configName = configName;
            }
            if (configId) {
                AppState.currentConfigId = configId;
                console.log('Set currentConfigId to:', configId);
            }
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
}

// 自动连接默认配置
async function autoConnectDefault() {
    try {
        const result = await invoke('get_default_config');
        if (result.success && result.data) {
            const config = result.data;
            console.log('Auto-connecting to default config:', config.name, 'id:', config.id);
            // 直接连接，不依赖表单元素
            await connectToServer(config.server_url, config.token, config.name, config.id);
        }
    } catch (e) {
        console.error('Auto-connect failed:', e);
    }
}

// 监听新消息
listen('new-message', (event) => {
    console.log('收到新消息:', event.payload);
    const newMessage = event.payload;
    // 新消息添加到列表开头
    AppState.messages.unshift(newMessage);
    // 保持最多100条消息
    if (AppState.messages.length > 100) {
        AppState.messages = AppState.messages.slice(0, 100);
    }
    renderMessages();

    // 发送系统通知
    if (AppState.settings.enable_notifications) {
        invoke('send_notification', {
            title: newMessage.title || '新消息',
            body: newMessage.message
        }).catch(e => console.error('发送通知失败:', e));
    }
});

// 加载应用设置
async function loadAppSettings() {
    try {
        const result = await invoke('get_app_settings');
        if (result.success) {
            AppState.settings = result.data;
            document.getElementById('enableAutostart').checked = AppState.settings.enable_autostart;
            document.getElementById('minimizeToTray').checked = AppState.settings.minimize_to_tray;
            document.getElementById('silentStart').checked = AppState.settings.silent_start;
            document.getElementById('enableNotifications').checked = AppState.settings.enable_notifications;
        }
    } catch (e) {
        console.error('加载应用设置错误:', e);
    }
}

// 保存应用设置
async function saveAppSettings() {
    try {
        const settings = {
            enable_autostart: document.getElementById('enableAutostart').checked,
            minimize_to_tray: document.getElementById('minimizeToTray').checked,
            silent_start: document.getElementById('silentStart').checked,
            enable_notifications: document.getElementById('enableNotifications').checked
        };

        // 保存设置到文件
        const result = await invoke('update_app_settings', { settings });
        if (!result.success) {
            alert(result.error || '保存设置失败');
            return;
        }

        // 实际启用/禁用开机启动
        const autostartResult = await invoke('toggle_autostart', { enabled: settings.enable_autostart });
        if (!autostartResult.success) {
            console.error('开机启动设置失败:', autostartResult.error);
            alert(`设置已保存，但开机启动功能失败: ${autostartResult.error}`);
            return;
        }

        AppState.settings = settings;
        alert('设置保存成功');
    } catch (e) {
        console.error('保存应用设置错误:', e);
        alert(`保存设置错误: ${e}`);
    }
}

// 请求通知权限
function requestNotificationPermission() {
    if ('Notification' in window && Notification.permission === 'default') {
        Notification.requestPermission();
    }
}

// 初始渲染和事件绑定
document.addEventListener('DOMContentLoaded', () => {
    render();
    setupTooltips();
    loadConfigs();
    loadAppSettings();

    // 绑定连接按钮事件
    const connectBtn = document.getElementById('connectBtn');
    if (connectBtn) {
        connectBtn.addEventListener('click', async () => {
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
    }

    // 绑定断开连接按钮事件
    const disconnectBtn = document.getElementById('disconnectBtn');
    if (disconnectBtn) {
        disconnectBtn.addEventListener('click', async () => {
            try {
                await invoke('disconnect_gotify');
                AppState.connected = false;
                AppState.messages = [];
                AppState.configName = null;
                updateUIState({ error: null });
            } catch (e) {
                updateUIState({ error: `断开连接错误: ${e}` });
            }
        });
    }

    // 加载侧边栏状态
    try {
        const savedState = localStorage.getItem('sidebarCollapsed');
        if (savedState === 'true') {
            AppState.sidebarCollapsed = true;
            const sidebar = document.getElementById('sidebar');
            const mainContent = document.querySelector('.main-content');
            if (sidebar) {
                sidebar.classList.add('collapsed');
            }
            if (mainContent) {
                mainContent.style.marginLeft = '50px';
            }
        }
    } catch (e) {
        console.error('Failed to load sidebar state:', e);
    }

    // 延迟自动连接，确保配置已加载
    setTimeout(() => {
        autoConnectDefault();
    }, 500);
});

// 暴露函数到全局作用域
window.switchPage = switchPage;
window.toggleSidebar = toggleSidebar;
window.useConfig = useConfig;
window.deleteConfig = deleteConfig;
window.deleteMessage = deleteMessage;
window.refreshMessages = refreshMessages;
window.showAddConfigModal = showAddConfigModal;
window.closeConfigModal = closeConfigModal;
window.saveConfigFromModal = saveConfigFromModal;
window.editConfig = editConfig;
window.loadMoreMessages = loadMoreMessages;
window.saveAppSettings = saveAppSettings;