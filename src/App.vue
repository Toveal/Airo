<template>
  <div class="app-shell" :data-theme="theme">
    <TitleBar :is-dark="theme === 'dark'" @toggle-theme="toggleTheme" />

    <div class="app-body">
      <nav class="views">
        <button v-for="view in views" :key="view.id" class="view-tab"
          :class="{ active: activeView === view.id }" @click="activeView = view.id">
          {{ t(view.labelKey) }}
        </button>
      </nav>

      <!-- The monitor tabs live outside .content on purpose: anything inside
           that scroll area is pushed out of sight once the settings outgrow
           the window, and switching monitors has to stay reachable. -->
      <div v-if="activeView === 'monitors' && monitors.length > 1" class="subheader">
        <MonitorTabs :monitors="monitors" :profiles="profiles" :active-id="activeId"
          :live-monitor-id="liveMonitorId" @select="activeId = $event" />
      </div>

      <div class="content">
        <template v-if="activeView === 'monitors'">
          <Transition name="slide-fade" mode="out-in">
            <MonitorSettings v-if="activeMonitor && activeProfile" :key="activeId"
              :monitor="activeMonitor" :profile="activeProfile"
              @update:channel="(slot, ch, v) => setChannel(activeId, slot, ch, v)"
              @update:color="(slot, rgb) => setColor(activeId, slot, rgb)"
              @update:enabled="(v) => setEnabled(activeId, v)" />

            <div v-else class="empty">
              <p>{{ t('monitors.empty') }}</p>
              <button class="btn btn-secondary" @click="refreshMonitors">{{ t('actions.refresh') }}</button>
            </div>
          </Transition>
        </template>

        <SettingsPanel v-else :preferences="preferences" :autostart="autostart"
          @update:preferences="setPreferences" @update:autostart="setAutostart" />
      </div>

      <div class="actions">
        <span class="status" :class="saveState">{{ statusText }}</span>

        <div class="actions-right">
          <button v-if="activeView === 'monitors' && activeMonitor" class="btn btn-ghost"
            @click="resetMonitor(activeId)">
            {{ t('actions.reset') }}
          </button>
          <button class="btn btn-secondary" @click="restoreColors"
            :title="t('actions.systemColorsHint')">
            {{ t('actions.systemColors') }}
          </button>
          <button v-if="activeView === 'monitors' && activeMonitor" class="btn btn-primary"
            @click="previewMonitor(activeId)" :title="t('actions.applyNowHint')">
            {{ t('actions.applyNow') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import TitleBar from './components/TitleBar.vue';
import MonitorTabs from './components/MonitorTabs.vue';
import MonitorSettings from './components/MonitorSettings.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import { useTheme } from './composables/useTheme.js';
import { useAiro } from './composables/useAiro.js';

const { t } = useI18n();
const { theme, toggleTheme } = useTheme();

const {
  monitors,
  profiles,
  preferences,
  autostart,
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
  setPreferences,
  setAutostart,
  refreshMonitors,
} = useAiro();

const views = [
  { id: 'monitors', labelKey: 'views.monitors' },
  { id: 'settings', labelKey: 'views.settings' },
];
const activeView = ref('monitors');

const statusText = computed(() => {
  switch (saveState.value) {
    case 'saving':
      return t('status.saving');
    case 'saved':
      return t('status.saved');
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

.views {
  display: flex;
  gap: 2px;
  padding: 14px 24px 0;
  border-bottom: 1px solid var(--border-primary);
}

.view-tab {
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  font-family: 'Outfit', sans-serif;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color var(--transition-fast), border-color var(--transition-fast);
}

.view-tab:hover {
  color: var(--text-secondary);
}

.view-tab.active {
  color: var(--text-primary);
  font-weight: 600;
  border-bottom-color: var(--accent);
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
  .views,
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
