import { onUnmounted, ref, unref, watch, type Ref } from 'vue';
import { authService } from '@/services/auth';
import { webSocketService } from '@/services/websocket';

interface UseWebSocketOptions {
  enabled?: Ref<boolean> | boolean;
  onInstanceChange?: (data: unknown) => void;
  onNotification?: (data: unknown) => void;
}

export function useWebSocket({ enabled = true, onInstanceChange, onNotification }: UseWebSocketOptions = {}) {
  const isConnected = ref(webSocketService.isConnected());
  let listenersAttached = false;

  const handleInstanceChange = (event: Event) => {
    if (!onInstanceChange) return;
    const customEvent = event as CustomEvent;
    onInstanceChange(customEvent.detail);
  };

  const handleNotification = (event: Event) => {
    if (!onNotification) return;
    const customEvent = event as CustomEvent;
    onNotification(customEvent.detail);
  };

  const attachListeners = () => {
    if (listenersAttached) return;
    if (onInstanceChange) {
      window.addEventListener('instance_change', handleInstanceChange as EventListener);
      listenersAttached = true;
    }
    if (onNotification) {
      window.addEventListener('notification', handleNotification as EventListener);
      listenersAttached = true;
    }
  };

  const detachListeners = () => {
    if (!listenersAttached) return;
    if (onInstanceChange) {
      window.removeEventListener('instance_change', handleInstanceChange as EventListener);
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

