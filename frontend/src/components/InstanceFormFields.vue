<script setup lang="ts">
import { computed } from 'vue';
import { Upload } from 'lucide-vue-next';
import type { AuthBackend, InstanceBase } from '@/types/api';
import { AuthType, LoaderType } from '@/types/api';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';

const props = withDefaults(
    defineProps<{
        formData: InstanceBase;
        minecraftVersions: string[];
        availableLoaders: string[];
        loaderVersions: string[];
        loadingMinecraftVersions?: boolean;
        loadingLoaders?: boolean;
        loadingLoaderVersions?: boolean;
        errors?: Record<string, string>;
        disabled?: boolean;
        uploadedFiles: FileList | null;
        showFileUpload?: boolean;
        idPrefix?: string;
    }>(),
    {
        loadingMinecraftVersions: false,
        loadingLoaders: false,
        loadingLoaderVersions: false,
        errors: () => ({}),
        disabled: false,
        uploadedFiles: null,
        showFileUpload: true,
        idPrefix: 'instance',
    },
);

const emit = defineEmits<{
    (event: 'update-field', field: keyof InstanceBase, value: string | LoaderType): void;
    (event: 'update-auth-field', field: keyof AuthBackend, value: string | AuthType): void;
    (event: 'file-drag', eventObj: DragEvent): void;
    (event: 'file-drop', eventObj: DragEvent): void;
    (event: 'file-input', eventObj: Event): void;
}>();

const isVanillaLoader = computed(() => props.formData.loader_name === LoaderType.VANILLA);
</script>

<template>
    <div class="space-y-5">
        <div class="grid gap-4 sm:grid-cols-2">
            <div class="space-y-2 sm:col-span-2">
                <Label :for="`${props.idPrefix}-name`">Instance Name *</Label>
                <Input :id="`${props.idPrefix}-name`" :model-value="props.formData.name" :disabled="props.disabled"
                    placeholder="Enter instance name"
                    @update:modelValue="(value) => emit('update-field', 'name', value?.toString() ?? '')" />
                <p v-if="props.errors?.name" class="text-sm">
                    {{ props.errors.name }}
                </p>
            </div>
            <div class="space-y-2">
                <Label>Minecraft Version *</Label>
                <Select :model-value="props.formData.minecraft_version || undefined"
                    :disabled="props.disabled || props.loadingMinecraftVersions"
                    @update:modelValue="(value) => emit('update-field', 'minecraft_version', value?.toString() ?? '')">
                    <SelectTrigger>
                        <SelectValue placeholder="Select version" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem v-for="version in props.minecraftVersions" :key="version" :value="version">
                            {{ version }}
                        </SelectItem>
                    </SelectContent>
                </Select>
                <p v-if="props.errors?.minecraft_version" class="text-sm">
                    {{ props.errors.minecraft_version }}
                </p>
                <p v-else-if="props.loadingMinecraftVersions" class="text-sm">
                    Loading versions...
                </p>
            </div>
            <div class="space-y-2">
                <Label>Mod Loader *</Label>
                <Select :model-value="props.formData.loader_name || undefined" :disabled="props.disabled ||
                    props.loadingLoaders ||
                    !props.formData.minecraft_version ||
                    props.availableLoaders.length === 0
                    " @update:modelValue="(value) =>
                        emit(
                            'update-field',
                            'loader_name',
                            (typeof value === 'string' && value.length ? value : LoaderType.VANILLA) as LoaderType,
                        )">
                    <SelectTrigger>
                        <SelectValue placeholder="Select loader" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem v-for="loader in props.availableLoaders" :key="loader" :value="loader">
                            {{ loader }}
                        </SelectItem>
                    </SelectContent>
                </Select>
                <p v-if="props.errors?.loader_name" class="text-sm">
                    {{ props.errors.loader_name }}
                </p>
                <p v-else-if="!props.formData.minecraft_version" class="text-sm">
                    Select a Minecraft version first.
                </p>
                <p v-else-if="props.availableLoaders.length === 0" class="text-sm">
                    No loaders available.
                </p>
            </div>
            <div class="space-y-2">
                <Label>Loader Version *</Label>
                <Select :model-value="props.formData.loader_version || undefined" :disabled="props.disabled ||
                    props.loadingLoaderVersions ||
                    !props.formData.loader_name ||
                    props.loaderVersions.length === 0 ||
                    isVanillaLoader
                    " @update:modelValue="(value) => emit('update-field', 'loader_version', value?.toString() ?? '')">
                    <SelectTrigger>
                        <SelectValue placeholder="Select version" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem v-for="version in props.loaderVersions" :key="version" :value="version">
                            {{ version }}
                        </SelectItem>
                    </SelectContent>
                </Select>
                <p v-if="props.errors?.loader_version" class="text-sm">
                    {{ props.errors.loader_version }}
                </p>
                <p v-else-if="!props.formData.loader_name" class="text-sm">
                    Select a loader first.
                </p>
                <p v-else-if="!isVanillaLoader && props.loaderVersions.length === 0" class="text-sm">
                    No versions available.
                </p>
            </div>
            <div class="space-y-2">
                <Label>Authentication Type *</Label>
                <Select :model-value="props.formData.auth_backend.type" :disabled="props.disabled"
                    @update:modelValue="(value) => emit('update-auth-field', 'type', (value as AuthType) ?? AuthType.OFFLINE)">
                    <SelectTrigger>
                        <SelectValue placeholder="Select authentication" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem :value="AuthType.OFFLINE">Offline</SelectItem>
                        <SelectItem :value="AuthType.MOJANG">Mojang</SelectItem>
                        <SelectItem :value="AuthType.TELEGRAM">Telegram</SelectItem>
                        <SelectItem :value="AuthType.ELY_BY">Ely.by</SelectItem>
                    </SelectContent>
                </Select>
                <p v-if="props.errors?.auth_type" class="text-sm">
                    {{ props.errors.auth_type }}
                </p>
            </div>
        </div>
        <div v-if="props.formData.auth_backend.type === AuthType.TELEGRAM" class="space-y-2">
            <Label :for="`${props.idPrefix}-auth-base-url`">Auth Base URL *</Label>
            <Input :id="`${props.idPrefix}-auth-base-url`" type="url"
                :model-value="props.formData.auth_backend.auth_base_url || ''" :disabled="props.disabled"
                placeholder="https://your-telegram-auth-server.com"
                @update:modelValue="(value) => emit('update-auth-field', 'auth_base_url', value?.toString() ?? '')" />
            <p v-if="props.errors?.auth_base_url" class="text-sm">
                {{ props.errors.auth_base_url }}
            </p>
        </div>
        <div v-if="props.formData.auth_backend.type === AuthType.ELY_BY" class="grid gap-4 sm:grid-cols-2">
            <div class="space-y-2">
                <Label :for="`${props.idPrefix}-client-id`">Client ID *</Label>
                <Input :id="`${props.idPrefix}-client-id`" :model-value="props.formData.auth_backend.client_id || ''"
                    :disabled="props.disabled" placeholder="Ely.by client ID"
                    @update:modelValue="(value) => emit('update-auth-field', 'client_id', value?.toString() ?? '')" />
                <p v-if="props.errors?.client_id" class="text-sm">
                    {{ props.errors.client_id }}
                </p>
            </div>
            <div class="space-y-2">
                <Label :for="`${props.idPrefix}-client-secret`">Client Secret *</Label>
                <Input :id="`${props.idPrefix}-client-secret`" type="password"
                    :model-value="props.formData.auth_backend.client_secret || ''" :disabled="props.disabled"
                    placeholder="Ely.by client secret"
                    @update:modelValue="(value) => emit('update-auth-field', 'client_secret', value?.toString() ?? '')" />
                <p v-if="props.errors?.client_secret" class="text-sm">
                    {{ props.errors.client_secret }}
                </p>
            </div>
        </div>
        <div v-if="props.showFileUpload" class="space-y-3">
            <Label>Upload Instance Files (optional)</Label>
            <div class="relative rounded-md border border-dashed p-6 text-center text-sm"
                @dragenter="(event) => emit('file-drag', event)" @dragleave="(event) => emit('file-drag', event)"
                @dragover="(event) => emit('file-drag', event)" @drop="(event) => emit('file-drop', event)">
                <input type="file" multiple class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
                    webkitdirectory="" :disabled="props.disabled" @change="(event) => emit('file-input', event)" />
                <Upload class="mx-auto mb-3 h-10 w-10" />
                <p>Drag a folder here or click to browse.</p>
            </div>
            <Alert v-if="props.uploadedFiles">
                <AlertDescription>{{ props.uploadedFiles.length }} file(s) selected</AlertDescription>
            </Alert>
        </div>
    </div>
</template>
