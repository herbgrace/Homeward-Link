export type ThemePreference = "system" | "light" | "dark";

const STORAGE_KEY = "homeward-theme";

function readStored(): ThemePreference {
    try {
        const t = localStorage.getItem(STORAGE_KEY);
        if (t === "light" || t === "dark") return t;
    } catch { /* storage unavailable */ }
    return "system";
}

export const theme = $state<{ preference: ThemePreference }>({ preference: readStored() });

export function setTheme(pref: ThemePreference) {
    theme.preference = pref;
    const root = document.documentElement;
    if (pref === "system") delete root.dataset.theme;
    else root.dataset.theme = pref;
    try {
        if (pref === "system") localStorage.removeItem(STORAGE_KEY);
        else localStorage.setItem(STORAGE_KEY, pref);
    } catch { /* storage unavailable */ }
}
