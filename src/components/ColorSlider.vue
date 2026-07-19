<template>
  <div class="color-slider">
    <div class="slider-header">
      <label class="slider-label">{{ label }}</label>
      <div class="slider-value-wrapper">
        <input type="number" class="slider-value-input" :value="modelValue" @input="onInputChange" min="0" :max="max"
          step="1" />
      </div>
    </div>
    <div class="slider-track-wrapper">
      <div class="slider-track" :style="trackStyle">
        <div class="slider-fill" :style="fillStyle"></div>
      </div>
      <input type="range" class="slider-input" :value="modelValue"
        @input="$emit('update:modelValue', parseInt($event.target.value))" min="0" :max="max" step="1" />
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  modelValue: { type: Number, required: true },
  label: { type: String, default: 'R' },
  channel: { type: String, default: 'r' },
  max: { type: Number, default: 255 },
});

const emit = defineEmits(['update:modelValue']);

const channelColors = {
  r: { start: 'rgb(255, 60, 60, 0.08)', end: 'rgb(255, 60, 60)', thumb: '#ff3c3c' },
  g: { start: 'rgb(50, 205, 100, 0.08)', end: 'rgb(50, 205, 100)', thumb: '#32cd64' },
  b: { start: 'rgb(60, 120, 255, 0.08)', end: 'rgb(60, 120, 255)', thumb: '#3c78ff' },
};

const colors = computed(() => channelColors[props.channel] || channelColors.r);
const percent = computed(() => (props.modelValue / props.max) * 100);

const trackStyle = computed(() => ({
  background: `linear-gradient(to right, ${colors.value.start}, ${colors.value.end})`,
}));

const fillStyle = computed(() => ({
  width: `${percent.value}%`,
  background: colors.value.end,
  opacity: 0.25,
}));

const onInputChange = (e) => {
  let val = parseInt(e.target.value);
  if (isNaN(val)) val = 0;
  if (val < 0) val = 0;
  if (val > props.max) val = props.max;
  emit('update:modelValue', val);
};
</script>

<style scoped>
.color-slider {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.slider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.slider-label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-tertiary);
  letter-spacing: 0.5px;
  text-transform: uppercase;
  font-family: 'JetBrains Mono', monospace;
}

.slider-value-wrapper {
  display: flex;
  align-items: center;
}

.slider-value-input {
  width: 44px;
  padding: 2px 6px;
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  font-weight: 500;
  color: var(--text-primary);
  background: var(--bg-input);
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-sm);
  text-align: center;
  outline: none;
  transition: border-color var(--transition-fast);
  -moz-appearance: textfield;
}

.slider-value-input::-webkit-outer-spin-button,
.slider-value-input::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.slider-value-input:focus {
  border-color: var(--border-focus);
}

.slider-track-wrapper {
  position: relative;
  height: 24px;
  display: flex;
  align-items: center;
}

.slider-track {
  position: absolute;
  width: 100%;
  height: 6px;
  border-radius: 3px;
  overflow: hidden;
}

.slider-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.05s linear;
}

.slider-input {
  position: relative;
  width: 100%;
  height: 6px;
  background: transparent;
  -webkit-appearance: none;
  appearance: none;
  cursor: pointer;
  z-index: 2;
}

.slider-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--bg-secondary);
  border: 2.5px solid v-bind('colors.thumb');
  cursor: pointer;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}

.slider-input::-webkit-slider-thumb:hover {
  transform: scale(1.15);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.slider-input::-webkit-slider-thumb:active {
  transform: scale(1.05);
}

.slider-input::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--bg-secondary);
  border: 2.5px solid v-bind('colors.thumb');
  cursor: pointer;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
}
</style>
