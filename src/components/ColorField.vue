<template>
  <div class="color-field">
    <div class="field-header">
      <div class="field-title">
        <span class="field-name">{{ title }}</span>
        <span class="field-hint">{{ hint }}</span>
      </div>

      <div class="field-inputs">
        <input class="hex-input" type="text" spellcheck="false" maxlength="7" :value="hexDraft"
          @input="onHexInput" @blur="hexDraft = hex" @keyup.enter="$event.target.blur()" />
        <label class="swatch" :style="{ background: css }">
          <input type="color" :value="hex" @input="onPickerInput" />
        </label>
      </div>
    </div>

    <div class="sliders">
      <ColorSlider :model-value="modelValue.r" label="R" channel="r"
        @update:model-value="(v) => emitChannel('r', v)" />
      <ColorSlider :model-value="modelValue.g" label="G" channel="g"
        @update:model-value="(v) => emitChannel('g', v)" />
      <ColorSlider :model-value="modelValue.b" label="B" channel="b"
        @update:model-value="(v) => emitChannel('b', v)" />
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue';
import ColorSlider from './ColorSlider.vue';
import { fromHex, toCss, toHex } from '../lib/color.js';

const props = defineProps({
  modelValue: { type: Object, required: true },
  title: { type: String, required: true },
  hint: { type: String, default: '' },
});

const emit = defineEmits(['update:channel', 'update:color']);

const hex = computed(() => toHex(props.modelValue));
const css = computed(() => toCss(props.modelValue));

// A separate draft value, so a half-typed `#0f` is not reset on every keystroke.
const hexDraft = ref(hex.value);
watch(hex, (value) => {
  if (document.activeElement?.classList.contains('hex-input')) return;
  hexDraft.value = value;
});

const emitChannel = (channel, value) => emit('update:channel', channel, value);

const onHexInput = (event) => {
  hexDraft.value = event.target.value;
  const parsed = fromHex(event.target.value);
  if (parsed) emit('update:color', parsed);
};

const onPickerInput = (event) => {
  const parsed = fromHex(event.target.value);
  if (parsed) emit('update:color', parsed);
};
</script>

<style scoped>
.color-field {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-card);
}

.field-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.field-title {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.field-name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.field-hint {
  font-size: 11.5px;
  color: var(--text-tertiary);
  line-height: 1.35;
}

.field-inputs {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.hex-input {
  width: 84px;
  padding: 5px 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  text-transform: uppercase;
  color: var(--text-primary);
  background: var(--bg-input);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--transition-fast);
}

.hex-input:focus {
  border-color: var(--border-focus);
}

.swatch {
  position: relative;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-primary);
  cursor: pointer;
  overflow: hidden;
  flex-shrink: 0;
  transition: transform var(--transition-fast);
}

.swatch:hover {
  transform: scale(1.06);
}

/* The native picker hides under the swatch: it brings the system palette and eyedropper. */
.swatch input[type='color'] {
  position: absolute;
  inset: 0;
  opacity: 0;
  width: 100%;
  height: 100%;
  border: none;
  padding: 0;
  cursor: pointer;
}

.sliders {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
</style>
