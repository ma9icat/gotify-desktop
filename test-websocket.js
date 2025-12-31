// 测试 WebSocket 连接的脚本
// 在浏览器控制台或 Node.js 中运行

const testWebSocket = (serverUrl, token) => {
    const wsUrl = `${serverUrl.replace('http', 'ws')}/stream?token=${token}`;

    console.log('正在连接到 WebSocket:', wsUrl);

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
        console.log('✓ WebSocket 连接成功');
        console.log('等待接收消息...');
    };

    ws.onmessage = (event) => {
        console.log('✓ 收到消息:', event.data);
        try {
            const message = JSON.parse(event.data);
            console.log('解析后的消息:', message);
        } catch (e) {
            console.log('无法解析为 JSON:', e);
        }
    };

    ws.onerror = (error) => {
        console.error('✗ WebSocket 错误:', error);
    };

    ws.onclose = (event) => {
        console.log('WebSocket 连接关闭:', event.code, event.reason);
    };

    // 10 秒后自动关闭
    setTimeout(() => {
        console.log('测试结束，关闭连接');
        ws.close();
    }, 10000);
};

// 使用示例（替换为你的服务器 URL 和 Token）
// testWebSocket('https://your-gotify-server.com', 'your-token');

console.log('请调用 testWebSocket(serverUrl, token) 来测试');