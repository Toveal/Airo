<template>
  <div class="panel">
    <label v-for="option in options" :key="option.key" class="option">
      <input type="checkbox" :checked="option.value" @change="option.onChange($event.target.checked)" />
      <span class="check"><svg viewBox="0 0 12 12" width="10" height="10">
          <path d="M2 6.2 4.6 8.8 10 3.4" fill="none" stroke="currentColor" stroke-width="1.8"
            stroke-linecap="round" stroke-linejoin="round" />
        </svg></span>
      <span class="text">
        <span class="title">{{ option.title }}</span>
        <span class="hint">{{ option.hint }}</span>
      </span>
    </label>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  preferences: { type: Object, required: true },
  autostart: { type: Boolean, required: true },
});

const emit = defineEmits(['update:preferences', 'update:autostart']);

const patch = (key) => (value) => emit('update:preferences', { [key]: value });

const options = computed(() => [
  {
    key: 'autostart',
    title: 'Start with Windows',
    hint: 'Airo starts minimized and begins tracking the cursor right away',
    value: props.autostart,
    onChange: (value) => emit('update:autostart', value),
  },
  {
    key: 'applyOnClick',
    title: 'Change colour on click',
    hint: 'The colour is applied on the first mouse click after you move to another '
      + 'monitor. Turn this off to change it the moment the cursor crosses the boundary '
      + '— more noticeable, but it forces every window to repaint on each crossing.',
    value: props.preferences.applyOnClick,
    onChange: patch('applyOnClick'),
  },
  {
    key: 'startHidden',
    title: 'Start minimized to tray',
    hint: 'The window is not shown on startup',
    value: props.preferences.startHidden,
    onChange: patch('startHidden'),
  },
]);
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 9px 10px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.option:hover {
  background: var(--bg-hover);
}

.option input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-top: 1px;
  border-radius: 4px;
  border: 1.5px solid var(--border-primary);
  background: var(--bg-input);
  color: transparent;
  flex-shrink: 0;
  transition: all var(--transition-fast);
}

.option input:checked + .check {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.option input:focus-visible + .check {
  outline: 2px solid var(--border-focus);
  outline-offset: 2px;
}

.text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.title {
  font-size: 13px;
  color: var(--text-primary);
}

.hint {
  font-size: 11.5px;
  color: var(--text-tertiary);
  line-height: 1.4;
}
</style>
