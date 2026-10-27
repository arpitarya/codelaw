interface Prefs {
  theme: string;
  currency: string;
}

const DEFAULTS: Prefs = { theme: "light", currency: "EUR" };

function loadPrefs(): Prefs {
  const raw = localStorage.getItem("prefs");
  if (raw) {
    try {
      return { ...DEFAULTS, ...(JSON.parse(raw) as Partial<Prefs>) };
    } catch {}
  }
  return DEFAULTS;
}

export { loadPrefs };
export type { Prefs };
