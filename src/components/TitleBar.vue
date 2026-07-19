<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      <div class="app-icon">
        <img src="/airo.svg" alt="Airo" width="16" height="16" />
      </div>
      <span class="app-title">Airo</span>
    </div>

    <div class="titlebar-center" data-tauri-drag-region></div>

    <div class="titlebar-right">
      <!-- Theme toggle -->
      <button class="titlebar-btn theme-toggle" @click="$emit('toggle-theme')"
        :title="isDark ? 'Light theme' : 'Dark theme'">
        <svg v-if="isDark" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5" />
          <path
            d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
        </svg>
        <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" />
        </svg>
      </button>

      <!-- Minimize -->
      <button class="titlebar-btn" @click="minimizeWindow" title="Minimize">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
        </svg>
      </button>

      <!-- Maximize / Restore -->
      <button class="titlebar-btn" @click="toggleMaximize" :title="isMaximized ? 'Restore' : 'Maximize'">
        <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
          <rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.2" />
        </svg>
        <svg v-else width="12" height="12" viewBox="0 0 12 12">
          <rect x="3.5" y="0.5" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.1" />
          <rect x="0.5" y="3.5" width="8" height="8" fill="var(--bg-titlebar)" stroke="currentColor"
            stroke-width="1.1" />
        </svg>
      </button>

      <!-- Close -->
      <button class="titlebar-btn close-btn" @click="closeWindow" title="Close">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M1 1l10 10M11 1L1 11" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue';

defineProps({
  isDark: Boolean,
});
defineEmits(['toggle-theme']);

const isMaximized = ref(false);

import { getCurrentWindow } from '@tauri-apps/api/window';

const minimizeWindow = async () => {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.minimize();
    console.log('minimize');
  } catch (e) {
    console.error(e);
  }
};

const toggleMaximize = async () => {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.toggleMaximize();
    isMaximized.value = !isMaximized.value;
    console.log('toggle maximize');
  } catch (e) {
    console.error(e);
  }
};

const closeWindow = async () => {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.hide(); // Hide to tray instead of closing
    console.log('close (hide to tray)');
  } catch (e) {
    console.error(e);
  }
};

// Listen for maximize state changes
let unlisten = null;
onMounted(async () => {
  try {
    const appWindow = getCurrentWindow();
    isMaximized.value = await appWindow.isMaximized();
    unlisten = await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  } catch (e) {
    console.error(e);
  }
});

onUnmounted(() => {
  if (unlisten) unlisten();
});
</script>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 4px 0 12px;
  background: var(--bg-titlebar);
  border-bottom: 1px solid var(--border-primary);
  backdrop-filter: blur(20px);
  -webkit-app-region: drag;
  flex-shrink: 0;
  z-index: 100;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  -webkit-app-region: no-drag;
  pointer-events: none;
}

.app-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.3px;
}

.titlebar-center {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 1px;
  -webkit-app-region: no-drag;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 30px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.titlebar-btn:hover {
  background: var(--titlebar-btn-hover);
  color: var(--text-primary);
}

.titlebar-btn:active {
  background: var(--bg-active);
}

.close-btn:hover {
  background: var(--titlebar-close-hover);
  color: var(--titlebar-close-text);
}

.theme-toggle {
  margin-right: 8px;
  width: 30px;
  height: 28px;
  border-radius: var(--radius-sm);
}

.theme-toggle:hover {
  background: var(--accent-soft);
  color: var(--accent);
}
</style>
