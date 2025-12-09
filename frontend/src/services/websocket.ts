class WebSocketService {
  private ws: WebSocket | null = null;

  private reconnectAttempts = 0;

  private readonly maxReconnectAttempts = 5;

  private readonly reconnectDelay = 1000;

  private isConnecting = false;

  private shouldReconnect = true;

  async connect(token: string): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    if (this.isConnecting) {
      throw new Error('Connection already in progress');
    }

    this.isConnecting = true;
    this.shouldReconnect = true;

    const wsUrl = import.meta.env.VITE_API_BASE_URL
      ? `${import.meta.env.VITE_API_BASE_URL.replace('http', 'ws')}/api/v1/ws`
      : `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/v1/ws`;

    this.ws = new WebSocket(`${wsUrl}?token=${encodeURIComponent(token)}`);

    this.ws.onopen = () => {
      this.isConnecting = false;
      this.reconnectAttempts = 0;
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleMessage(data);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = (event) => {
      this.isConnecting = false;
      this.ws = null;

      if (this.shouldReconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts += 1;
        setTimeout(() => {
          if (this.shouldReconnect && token) {
            this.connect(token).catch((err) => {
              console.error('WebSocket reconnect failed:', err);
            });
          }
        }, this.reconnectDelay * this.reconnectAttempts);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.isConnecting = false;
    };
  }

  disconnect(): void {
    this.shouldReconnect = false;
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private handleMessage(data: any): void {
    switch (data.type) {
      case 'instance_created':
      case 'instance_updated':
      case 'instance_deleted':
        window.dispatchEvent(new CustomEvent('instance_change', { detail: data }));
        break;
      case 'notification':
        window.dispatchEvent(new CustomEvent('notification', { detail: data }));
        break;
      default:
        console.log('Unknown message type:', data.type);
    }
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

export const webSocketService = new WebSocketService();

