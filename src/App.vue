<template>
  <div class="app-shell" :data-theme="theme">
    <TitleBar :is-dark="theme === 'dark'" @toggle-theme="toggleTheme" />

    <div class="app-body">
      <!-- The monitor tabs live outside .content on purpose: anything inside
           that scroll area is pushed out of sight once the settings outgrow
           the window, and switching monitors has to stay reachable. -->
      <div v-if="monitors.length > 1" class="subheader">
        <MonitorTabs :monitors="monitors" :profiles="profiles" :active-id="activeId"
          :live-monitor-id="liveMonitorId" @select="activeId = $event" />
      </div>

      <div class="content">
        <Transition name="slide-fade" mode="out-in">
          <MonitorSettings v-if="activeMonitor && activeProfile" :key="activeId"
            :monitor="activeMonitor" :profile="activeProfile"
            @update:channel="(slot, ch, v) => setChannel(activeId, slot, ch, v)"
            @update:color="(slot, rgb) => setColor(activeId, slot, rgb)"
            @update:enabled="(v) => setEnabled(activeId, v)" />

          <div v-else class="empty">
            <p>No monitors found</p>
            <button class="btn btn-secondary" @click="refreshMonitors">Refresh list</button>
          </div>
        </Transition>
      </div>

      <div class="actions">
        <span class="status" :class="saveState">{{ statusText }}</span>

        <div class="actions-right">
          <button v-if="activeMonitor" class="btn btn-ghost" @click="resetMonitor(activeId)">
            Reset
          </button>
          <button class="btn btn-secondary" @click="restoreColors"
            title="Restore the colours that were in place before Airo started">
            System colors
          </button>
          <button v-if="activeMonitor" class="btn btn-primary" @click="previewMonitor(activeId)"
            title="Apply this monitor's colours right now">
            Apply now
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue';
import TitleBar from './components/TitleBar.vue';
import MonitorTabs from './components/MonitorTabs.vue';
import MonitorSettings from './components/MonitorSettings.vue';
import { useTheme } from './composables/useTheme.js';
import { useAiro } from './composables/useAiro.js';

const { theme, toggleTheme } = useTheme();

const {
  monitors,
  profiles,
  activeId,
  activeMonitor,
  activeProfile,
  liveMonitorId,
  saveState,
  lastError,
  init,
  setChannel,
  setColor,
  setEnabled,
  resetMonitor,
  previewMonitor,
  restoreColors,
  refreshMonitors,
} = useAiro();

const statusText = computed(() => {
  switch (saveState.value) {
    case 'saving':
      return 'Saving…';
    case 'saved':
      return 'Saved';
    case 'error':
      // The message comes from Rust as is — technical diagnostics.
      return lastError.value ?? '';
    default:
      return '';
  }
});

onMounted(init);
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

.subheader {
  flex-shrink: 0;
  padding: 14px 24px 0;
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

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  color: var(--text-tertiary);
  font-size: 14px;
}

.actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 24px;
  border-top: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.actions-right {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.status {
  font-size: 12px;
  color: var(--text-tertiary);
  transition: color var(--transition-fast);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status.error {
  color: #e05252;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: var(--radius-md);
  font-family: 'Outfit', sans-serif;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-ghost {
  background: transparent;
  color: var(--text-tertiary);
}

.btn-ghost:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-hover);
}

.btn-primary {
  background: var(--accent);
  color: #ffffff;
  font-weight: 600;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
  box-shadow: 0 2px 12px var(--accent-glow);
}

@media (max-width: 560px) {
  .subheader,
  .content,
  .actions {
    padding-left: 14px;
    padding-right: 14px;
  }

  .btn {
    padding: 7px 12px;
    font-size: 12px;
  }
}
</style>
