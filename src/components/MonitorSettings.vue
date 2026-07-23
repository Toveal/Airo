<template>
  <div class="settings">
    <div class="monitor-head">
      <div class="monitor-title">
        <h2>{{ monitor.label }}</h2>
        <span class="monitor-meta">
          {{ monitor.deviceName }} · {{ monitor.width }} × {{ monitor.height }}
          <template v-if="monitor.isPrimary"> · {{ t('monitors.primary') }}</template>
        </span>
      </div>

      <label class="toggle"
        :title="profile.enabled ? t('monitors.disableHint') : t('monitors.enableHint')">
        <input type="checkbox" :checked="profile.enabled"
          @change="$emit('update:enabled', $event.target.checked)" />
        <span class="toggle-track"><span class="toggle-thumb" /></span>
        <span class="toggle-label">{{ profile.enabled ? t('monitors.enabled') : t('monitors.disabled') }}</span>
      </label>
    </div>

    <div class="layout" :class="{ dimmed: !profile.enabled }">
      <div class="fields">
        <ColorField :model-value="profile.highlight" :title="t('colors.outline')"
          @update:channel="(ch, v) => $emit('update:channel', 'highlight', ch, v)"
          @update:color="(rgb) => $emit('update:color', 'highlight', rgb)" />

        <ColorField :model-value="profile.hotlight" :title="t('colors.fill')"
          @update:channel="(ch, v) => $emit('update:channel', 'hotlight', ch, v)"
          @update:color="(rgb) => $emit('update:color', 'hotlight', rgb)" />
      </div>

      <div class="preview-panel">
        <ColorPreview :outline="profile.highlight" :fill="profile.hotlight" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { useI18n } from 'vue-i18n';
import ColorField from './ColorField.vue';
import ColorPreview from './ColorPreview.vue';

const { t } = useI18n();

defineProps({
  monitor: { type: Object, required: true },
  profile: { type: Object, required: true },
});

defineEmits(['update:channel', 'update:color', 'update:enabled']);
</script>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.monitor-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
}

.monitor-title h2 {
  font-size: 17px;
  font-weight: 600;
  color: var(--text-primary);
}

.monitor-meta {
  font-size: 11.5px;
  color: var(--text-tertiary);
}

.layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(260px, 46%);
  gap: 18px;
  align-items: start;
  transition: opacity var(--transition-normal);
}

.layout.dimmed {
  opacity: 0.45;
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.preview-panel {
  min-width: 0;
}

@media (max-width: 820px) {
  .layout {
    grid-template-columns: 1fr;
  }
}

/* Toggle */
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12.5px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.toggle input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.toggle-track {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: var(--bg-slider-track);
  padding: 2px;
  transition: background var(--transition-fast);
}

.toggle-thumb {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--bg-secondary);
  box-shadow: var(--shadow-sm);
  transition: transform var(--transition-fast);
}

.toggle input:checked + .toggle-track {
  background: var(--accent);
}

.toggle input:checked + .toggle-track .toggle-thumb {
  transform: translateX(16px);
}

.toggle input:focus-visible + .toggle-track {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
}
</style>
