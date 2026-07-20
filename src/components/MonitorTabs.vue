<template>
  <div class="monitor-tabs">
    <button v-for="monitor in monitors" :key="monitor.id" class="monitor-tab"
      :class="{ active: monitor.id === activeId, live: monitor.id === liveMonitorId }"
      :title="`${monitor.label} — ${monitor.deviceName}`" @click="$emit('select', monitor.id)">
      <span class="tab-dot" :style="dotStyle(monitor.id)" />
      <span class="tab-name">{{ monitor.label }}</span>
      <span v-if="monitor.isPrimary" class="tab-badge">main</span>
      <!-- The "cursor is here" dot makes it obvious which tab belongs to the
           monitor the user is currently working on. -->
      <span v-if="monitor.id === liveMonitorId" class="tab-live" title="cursor is here" />
    </button>
  </div>
</template>

<script setup>
import { toCss } from '../lib/color.js';

const props = defineProps({
  monitors: { type: Array, required: true },
  profiles: { type: Object, required: true },
  activeId: { type: String, default: null },
  liveMonitorId: { type: String, default: null },
});

defineEmits(['select']);

const dotStyle = (id) => {
  const profile = props.profiles[id];
  if (!profile) return { background: 'var(--text-tertiary)' };
  return {
    background: toCss(profile.highlight),
    opacity: profile.enabled ? 1 : 0.3,
  };
};
</script>

<style scoped>
.monitor-tabs {
  display: flex;
  gap: 4px;
  padding: 4px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  overflow-x: auto;
}

.monitor-tab {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-family: 'Outfit', sans-serif;
  font-size: 13px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  position: relative;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
  justify-content: center;
}

.monitor-tab:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.monitor-tab.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-weight: 600;
  box-shadow: var(--shadow-sm);
}

.tab-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: all var(--transition-fast);
}

.tab-name {
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--accent-soft);
  color: var(--accent);
  flex-shrink: 0;
}

.tab-live {
  position: absolute;
  top: 5px;
  right: 6px;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 0 0 var(--accent-glow);
  animation: pulse 1.8s ease-out infinite;
}

@keyframes pulse {
  0% { box-shadow: 0 0 0 0 var(--accent-glow); }
  70% { box-shadow: 0 0 0 6px transparent; }
  100% { box-shadow: 0 0 0 0 transparent; }
}

@media (max-width: 560px) {
  .monitor-tab {
    padding: 7px 10px;
    font-size: 12px;
  }

  .tab-badge {
    display: none;
  }
}
</style>
