const clamp = (n) => Math.min(255, Math.max(0, Math.round(n)));

export function toHex({ r, g, b }) {
  return `#${[r, g, b].map((c) => clamp(c).toString(16).padStart(2, '0')).join('')}`;
}

/**
 * Parses `#rgb` / `#rrggbb`; the leading hash is optional.
 * @returns {{r:number,g:number,b:number}|null} null when the string is not a colour.
 */
export function fromHex(input) {
  const hex = String(input).trim().replace(/^#/, '');

  if (/^[0-9a-f]{3}$/i.test(hex)) {
    return {
      r: parseInt(hex[0] + hex[0], 16),
      g: parseInt(hex[1] + hex[1], 16),
      b: parseInt(hex[2] + hex[2], 16),
    };
  }

  if (/^[0-9a-f]{6}$/i.test(hex)) {
    return {
      r: parseInt(hex.slice(0, 2), 16),
      g: parseInt(hex.slice(2, 4), 16),
      b: parseInt(hex.slice(4, 6), 16),
    };
  }

  return null;
}

export function toCss({ r, g, b }) {
  return `rgb(${r}, ${g}, ${b})`;
}

export function toRgba({ r, g, b }, alpha) {
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}
