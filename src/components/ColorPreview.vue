<template>
  <div class="preview">
    <div class="preview-label">{{ t('preview.title') }}</div>

    <!-- This shows the drag selection rectangle on purpose: in a modern
         Explorer it is the only thing that actually gets recoloured. Selected
         files are drawn by the Explorer theme and ignore the system colours,
         which is why the tiles under the rectangle stay neutral. -->
    <div class="canvas">
      <div class="tiles">
        <div v-for="file in files" :key="file" class="tile">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <span>{{ file }}</span>
        </div>
      </div>

      <div class="band" :style="bandStyle" />

      <svg class="cursor" :style="cursorStyle" width="16" height="20" viewBox="0 0 16 20">
        <path d="M1 1l12 11.5-5.6.4 3 5.5-2.4 1.3-3-5.6-3.9 4z" fill="#fff" stroke="#1a1a2e"
          stroke-width="1.1" stroke-linejoin="round" />
      </svg>
    </div>

    <div class="legend">
      <div class="legend-item">
        <span class="legend-swatch" :style="{ background: outlineCss }" />
        <span class="legend-name">{{ t('preview.outline') }}</span>
        <span class="legend-value">{{ outlineHex }}</span>
      </div>
      <div class="legend-item">
        <span class="legend-swatch" :style="{ background: fillCss }" />
        <span class="legend-name">{{ t('preview.fill') }}</span>
        <span class="legend-value">{{ fillHex }}</span>
      </div>
    </div>

    <p class="note">{{ t('preview.note') }}</p>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { toCss, toHex, toRgba } from '../lib/color.js';

// tm() returns the raw message array and rt() resolves each entry — the
// supported way of working with lists in vue-i18n.
const { t, tm, rt } = useI18n();

const props = defineProps({
  /** COLOR_HIGHLIGHT — the selection rectangle's outline. */
  outline: { type: Object, required: true },
  /** COLOR_HOTLIGHT — its translucent fill. */
  fill: { type: Object, required: true },
});

/** Windows draws the rectangle translucent; the API never reports the value. */
const FILL_ALPHA = 0.35;

// Rectangle geometry in percent: the cursor drags it from the top left corner.
const BAND = { left: 10, top: 16, width: 62, height: 56 };

const files = computed(() => tm('preview.files').map((file) => rt(file)));

const outlineCss = computed(() => toCss(props.outline));
const fillCss = computed(() => toCss(props.fill));
const outlineHex = computed(() => toHex(props.outline).toUpperCase());
const fillHex = computed(() => toHex(props.fill).toUpperCase());

const bandStyle = computed(() => ({
  left: `${BAND.left}%`,
  top: `${BAND.top}%`,
  width: `${BAND.width}%`,
  height: `${BAND.height}%`,
  borderColor: outlineCss.value,
  background: toRgba(props.fill, FILL_ALPHA),
}));

const cursorStyle = computed(() => ({
  left: `${BAND.left + BAND.width}%`,
  top: `${BAND.top + BAND.height}%`,
}));
</script>

<style scoped>
.preview {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.preview-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--text-tertiary);
}

.canvas {
  position: relative;
  border: 1px solid var(--border-primary);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  box-shadow: var(--shadow-card);
  overflow: hidden;
  padding: 12px;
}

.tiles {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px 6px;
}

.tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  color: var(--text-tertiary);
  font-size: 10px;
  text-align: center;
}

.tile span {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.band {
  position: absolute;
  border: 1px solid;
  border-radius: 1px;
  pointer-events: none;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.cursor {
  position: absolute;
  pointer-events: none;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.35));
}

.legend {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
}

.legend-swatch {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  border: 1px solid var(--border-primary);
}

.legend-name {
  color: var(--text-secondary);
}

.legend-value {
  font-family: 'JetBrains Mono', monospace;
  color: var(--text-tertiary);
}

.note {
  font-size: 11px;
  line-height: 1.4;
  color: var(--text-tertiary);
}
</style>
