<script setup lang="ts">
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Monitor, Terminal, BookOpen } from 'lucide-vue-next';
import GithubIcon from '@/components/icons/GithubIcon.vue';
import AppleIcon from '@/components/icons/AppleIcon.vue';

const launcherName = import.meta.env.VITE_LAUNCHER_NAME || 'Potato Launcher';
const githubUrl = import.meta.env.VITE_GITHUB_URL;

const getFilename = (os: 'windows' | 'macos' | 'linux') => {
  const lowerName = launcherName.toLowerCase().replace(/ /g, '_');
  switch (os) {
    case 'windows':
      return `${launcherName}.exe`;
    case 'macos':
      return `${launcherName}.dmg`;
    case 'linux':
      return lowerName;
  }
};

const downloads = [
  {
    name: 'Windows',
    icon: Monitor,
    url: `/launcher/${getFilename('windows')}`,
    description: 'Download for Windows (x64)',
  },
  {
    name: 'macOS',
    icon: AppleIcon,
    url: `/launcher/${getFilename('macos')}`,
    description: 'Download for macOS (Universal)',
  },
  {
    name: 'Linux',
    icon: Terminal,
    url: `/launcher/${getFilename('linux')}`,
    description: 'Download for Linux (x64)',
  },
];
</script>

<template>
  <div class="min-h-screen bg-background flex flex-col items-center justify-center p-4">
    <Card class="w-full max-w-4xl">
      <CardHeader class="text-center space-y-4">
        <div class="flex justify-center mb-4">
          <img src="/favicon.ico" alt="Logo" class="w-24 h-24" />
        </div>
        <CardTitle class="text-4xl font-bold">{{ launcherName }}</CardTitle>
        <CardDescription class="text-xl">
          Download the launcher for your operating system
        </CardDescription>
      </CardHeader>
      <CardContent class="grid grid-cols-1 md:grid-cols-3 gap-6 pt-8">
        <div v-for="os in downloads" :key="os.name"
          class="h-full border rounded-xl p-6 hover:bg-muted/50 transition-colors flex flex-col items-center text-center space-y-4">
          <component :is="os.icon" class="w-16 h-16 text-primary" />
          <div class="space-y-2">
            <h3 class="font-semibold text-xl">{{ os.name }}</h3>
            <p class="text-muted-foreground text-sm">{{ os.description }}</p>
          </div>
          <Button variant="outline" class="w-full mt-auto" as-child>
            <a :href="os.url" class="no-underline">Download</a>
          </Button>
        </div>
      </CardContent>
      <div class="p-6 text-center border-t mt-8 flex justify-center gap-6">
        <a v-if="githubUrl" :href="githubUrl" target="_blank" rel="noopener noreferrer"
          class="inline-flex items-center text-muted-foreground hover:text-foreground transition-colors">
          <GithubIcon class="w-5 h-5 mr-2" />
          View source on GitHub
        </a>
      </div>
    </Card>
  </div>
</template>
