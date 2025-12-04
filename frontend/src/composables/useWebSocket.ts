import { onUnmounted, ref, unref, watch, type Ref } from 'vue';
import { authService } from '@/services/auth';
import { webSocketService } from '@/services/websocket';

interface UseWebSocketOptions {
  enabled?: Ref<boolean> | boolean;
  onModpackChange?: (data: unknown) => void;
  onNotification?: (data: unknown) => void;
}

export function useWebSocket({ enabled = true, onModpackChange, onNotification }: UseWebSocketOptions = {}) {
  const isConnected = ref(webSocketService.isConnected());
  let listenersAttached = false;

  const handleModpackChange = (event: Event) => {
    if (!onModpackChange) return;
    const customEvent = event as CustomEvent;
    onModpackChange(customEvent.detail);
  };

  const handleNotification = (event: Event) => {
    if (!onNotification) return;
    const customEvent = event as CustomEvent;
    onNotification(customEvent.detail);
  };

  const attachListeners = () => {
    if (listenersAttached) return;
    if (onModpackChange) {
      window.addEventListener('modpack_change', handleModpackChange as EventListener);
      listenersAttached = true;
    }
    if (onNotification) {
      window.addEventListener('notification', handleNotification as EventListener);
      listenersAttached = true;
    }
  };

  const detachListeners = () => {
    if (!listenersAttached) return;
    if (onModpackChange) {
      window.removeEventListener('modpack_change', handleModpackChange as EventListener);
    }
    if (onNotification) {
      window.removeEventListener('notification', handleNotification as EventListener);
    }
    listenersAttached = false;
  };

  const startConnection = async () => {
    const token = authService.getToken();
    if (!token) {
      return;
    }

    try {
      await webSocketService.connect(token);
      isConnected.value = webSocketService.isConnected();
      attachListeners();
    } catch (err) {
      console.error('Failed to connect to WebSocket:', err);
    }
  };

  const stopConnection = () => {
    detachListeners();
    webSocketService.disconnect();
    isConnected.value = false;
  };

  const stopWatch = watch(
    () => Boolean(unref(enabled)),
    (shouldConnect) => {
      if (shouldConnect) {
        startConnection().catch((err) => console.error(err));
      } else {
        stopConnection();
      }
    },
    { immediate: true },
  );

  onUnmounted(() => {
    stopWatch();
    stopConnection();
  });

  return {
    isConnected,
  };
}

