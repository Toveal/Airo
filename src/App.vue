<template>
  <div class="app-shell" :data-theme="theme">
    <TitleBar :is-dark="theme === 'dark'" @toggle-theme="toggleTheme" />

    <div class="app-body">
      <div class="content">
        <p class="placeholder">Monitor settings go here</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import TitleBar from './components/TitleBar.vue';
import { useTheme } from './composables/useTheme.js';

const { theme, toggleTheme } = useTheme();
</script>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--bg-primary);
}

.app-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.placeholder {
  margin: auto;
  color: var(--text-tertiary);
  font-size: 14px;
}
</style>
