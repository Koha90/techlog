/**
 * Manages system, light, and dark color themes.
 */
(() => {
  const storageKey = "techlog.theme";
  const modes = new Set(["system", "light", "dark"]);

  /**
   * Reads the saved theme preference.
   *
   * Falls back to the system theme when storage
   * is unavailable or contains an invalid value.
   *
   * @returns {"system" | "light" | "dark"}
   */
  function loadTheme() {
    try {
      const saved = localStorage.getItem(storageKey);

      return modes.has(saved) ? saved : "system";
    } catch {
      return "system";
    }
  }

  /**
   * Applies the selected theme to the root element.
   *
   * @param {"system" | "light" | "dark"} mode
   */
  function applyTheme(mode) {
    const root = document.documentElement;

    if (mode === "system") {
      root.removeAttribute("data-theme");
      return;
    }

    root.dataset.theme = mode;
  }

  /**
   * Saves the theme preference when possible.
   *
   * @param {"system" | "light" | "dark"} mode
   */
  function saveTheme(mode) {
    try {
      if (mode === "system") {
        localStorage.removeItem(storageKey);
      } else {
        localStorage.setItem(storageKey, mode);
      }
    } catch {
      // The theme remains active for this page.
    }
  }

  const initialMode = loadTheme();

  applyTheme(initialMode);

  document.addEventListener("DOMContentLoaded", () => {
    const select = document.querySelector("#theme-select");

    if (!select) {
      return;
    }

    select.value = initialMode;

    select.addEventListener("change", () => {
      const mode = select.value;

      if (!modes.has(mode)) {
        return;
      }

      applyTheme(mode);
      saveTheme(mode);
    });
  });
})();
