import { ref, watch, onMounted } from 'vue';

const theme = ref('dark');

export function useTheme() {
  const toggleTheme = () => {
    theme.value = theme.value === 'dark' ? 'light' : 'dark';
    applyTheme();
    localStorage.setItem('airo-theme', theme.value);
  };

  const setTheme = (t) => {
    theme.value = t;
    applyTheme();
    localStorage.setItem('airo-theme', theme.value);
  };

  const applyTheme = () => {
    document.documentElement.setAttribute('data-theme', theme.value);
  };

  onMounted(() => {
    const saved = localStorage.getItem('airo-theme');
    if (saved) {
      theme.value = saved;
    } else {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      theme.value = prefersDark ? 'dark' : 'light';
    }
    applyTheme();
  });

  return {
    theme,
    toggleTheme,
    setTheme,
    isDark: () => theme.value === 'dark',
  };
}
