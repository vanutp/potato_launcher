<script setup lang="ts">
import { ref } from 'vue';
import { Loader2 } from 'lucide-vue-next';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import type { TokenRequest } from '@/types/auth';

const props = defineProps<{
  loading: boolean;
  error: string | null;
}>();

const emit = defineEmits<{
  (event: 'login', payload: TokenRequest): void;
}>();

const token = ref('');
const tokenError = ref('');

const handleSubmit = () => {
  if (!token.value.trim()) {
    tokenError.value = 'Access token is required';
    return;
  }

  tokenError.value = '';
  emit('login', { token: token.value.trim() });
};

const handleTokenChange = (value: string | number) => {
  token.value = String(value);
  if (tokenError.value) {
    tokenError.value = '';
  }
};
</script>

<template>
  <div class="flex min-h-screen items-center justify-center p-4">
    <Card class="w-full max-w-md">
      <CardHeader>
        <CardTitle>Instance Manager</CardTitle>
        <CardDescription>Sign in with your access token.</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <Alert v-if="props.error" variant="destructive">
          <AlertDescription>{{ props.error }}</AlertDescription>
        </Alert>
        <form class="space-y-4" @submit.prevent="handleSubmit">
          <div class="space-y-2">
            <Label for="token">Access Token *</Label>
            <Input id="token" type="password" :disabled="props.loading" :model-value="token" placeholder="Enter token"
              autocomplete="off" autocapitalize="off" spellcheck="false" @update:modelValue="handleTokenChange" />
            <p v-if="tokenError">{{ tokenError }}</p>
          </div>
          <Button type="submit" class="w-full" :disabled="props.loading || !token.trim()">
            <Loader2 v-if="props.loading" class="h-4 w-4 animate-spin" />
            <span v-else>Sign In</span>
          </Button>
        </form>
        <p class="text-center">
          Use the token provided when Potato Launcher provisioning completes.
        </p>
      </CardContent>
    </Card>
  </div>
</template>
