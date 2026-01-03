import { browser } from '$app/environment';
import { config } from './config.svelte';


export interface ThemeColors {
    background: string;
    primary: string;
    secondary: string;
    accent: string;
    text: string;
    border: string;
    [key: string]: string;
}

export interface ThemeOptions {
    radius: string;
}

export interface Theme {
    id: string;
    name: string;
    colors: ThemeColors;
    options: ThemeOptions;
}

type Result<T = void> = { success: true; data?: T } | { success: false; error: string };


const STORAGE_KEY_CUSTOM = 'custom_themes';

const DEFAULT_THEMES: Theme[] = [
    {
        id: 'default',
        name: 'Default',
        colors: {
            background: '#0b0c10',
            primary: '#15171e',
            secondary: '#1a1c24',
            accent: '#2b3540',
            text: '#c5c6c7',
            subtext: '#808080',
            border: '#1f2833',
            blue: '#5e81ac',
            mauve: '#b48ead',
            red: '#bf616a',
            green: '#a3be8c',
            gray: '#4c566a',
            pink: '#d08770',
            yellow: '#ebcb8b',
            orange: '#d08770',
            purple: '#9a76cc',
            teal: '#88c0d0',
            cyan: '#66fcf1',
            gold: '#f0c674'
        },
        options: { radius: '6px' }
    },
    {
        id: 'oled',
        name: 'OLED',
        colors: {
            background: 'oklch(0.02 0.01 240)',
            primary: 'oklch(0.04 0.01 245)',
            secondary: 'oklch(0.06 0.02 250)',
            accent: 'oklch(0.10 0.02 255)',
            text: 'oklch(0.90 0.01 240)',
            subtext: 'oklch(0.50 0.01 240)',
            border: 'oklch(0.12 0.02 250)',
            blue: 'oklch(0.60 0.18 250)',
            mauve: 'oklch(0.60 0.13 300)',
            red: 'oklch(0.60 0.18 25)',
            green: 'oklch(0.60 0.18 140)',
            gray: 'oklch(0.30 0.01 240)',
            pink: 'oklch(0.65 0.16 340)',
            yellow: 'oklch(0.80 0.18 100)',
            orange: 'oklch(0.75 0.18 70)',
            purple: 'oklch(0.60 0.15 320)',
            teal: 'oklch(0.60 0.15 190)',
            cyan: 'oklch(0.70 0.15 210)',
            gold: 'oklch(0.80 0.16 85)'
        },
        options: { radius: '6px' }
    },
    {
        id: 'catppuccin-frappe',
        name: 'Catppuccin Frappé',
        colors: {
            primary: '#414559',
            secondary: '#51576d',
            background: '#303446',
            accent: '#292c3c',
            text: '#c6d0f5',
            subtext: '#808080',
            border: '#232634',
            rosewater: '#f2d5cf',
            flamingo: '#eebebe',
            pink: '#f4b8e4',
            mauve: '#ca9ee6',
            red: '#e78284',
            maroon: '#ea999c',
            peach: '#ef9f76',
            yellow: '#e5c890',
            green: '#a6d189',
            teal: '#81c8be',
            sky: '#99d1db',
            sapphire: '#85c1dc',
            blue: '#8caaee',
            lavender: '#babbf1'
        },
        options: { radius: '0.5rem' }
    }
];


class ThemeManager {
    #selectedId = $state<string>('default');
    #customThemes = $state<Theme[]>([]);
    readonly all = $derived([...DEFAULT_THEMES, ...this.#customThemes]);
    readonly current = $derived(this.all.find((t) => t.id === this.#selectedId) ?? DEFAULT_THEMES[0]);

    constructor() {
        if (browser) {
            const storedCustom = localStorage.getItem(STORAGE_KEY_CUSTOM);

            if (storedCustom) {
                try {
                    this.#customThemes = JSON.parse(storedCustom);
                } catch (e) {
                    console.error('Failed to parse custom themes', e);
                }
            }
        }
    }
    init() {
        $effect(() => {
            if (!browser) return;

            config.theme = this.#selectedId;
            localStorage.setItem(STORAGE_KEY_CUSTOM, JSON.stringify(this.#customThemes));

            const root = document.documentElement;
            const theme = this.current;

            let cssText = '';

            for (const [key, value] of Object.entries(theme.colors)) {
                if (typeof value === 'string') {
                    cssText += `--color-${key}: ${value}; `;
                }
            }

            const radius = theme.options?.radius ?? '0px';
            cssText += `--radius: ${radius};`;

            root.style.cssText = cssText;
        });
    }

    set(id: string) {
        this.#selectedId = id;
    }

    add(newTheme: Theme): Result {
        const isDefault = DEFAULT_THEMES.some((t) => t.id === newTheme.id);
        if (isDefault) return { success: false, error: 'Cannot overwrite a default theme.' };

        if (!this.#validate(newTheme)) {
            return { success: false, error: 'Invalid theme structure.' };
        }
        const exists = this.#customThemes.some((t) => t.id === newTheme.id);
        if (exists) {
            this.#customThemes = this.#customThemes.map((t) => (t.id === newTheme.id ? newTheme : t));
        } else {
            this.#customThemes = [...this.#customThemes, newTheme];
        }

        return { success: true };
    }

    remove(id: string) {
        this.#customThemes = this.#customThemes.filter((t) => t.id !== id);
        if (this.#selectedId === id) {
            this.#selectedId = 'default';
        }
    }


    export(id: string): Result {
        if (!browser) return { success: false, error: 'Environment does not support export.' };
        const theme = this.all.find((t) => t.id === id);
        if (!theme) return { success: false, error: 'Theme not found.' };

        try {
            const blob = new Blob([JSON.stringify(theme, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const link = document.createElement('a');

            link.href = url;
            link.download = `${theme.name.toLowerCase().replace(/\s+/g, '-')}-theme.json`;

            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(url);

            return { success: true };
        } catch (e) {
            return { success: false, error: 'Failed to create export blob.' };
        }
    }

    async import(file: File): Promise<Result<Theme>> {
        if (!browser) return { success: false, error: 'Environment does not support import.' };

        try {
            const text = await file.text();
            const importedTheme = JSON.parse(text);

            const result = this.add(importedTheme);
            if (!result.success) return { success: false, error: result.error };

            this.set(importedTheme.id);
            return { success: true, data: importedTheme };
        } catch (e) {
            return { success: false, error: 'Failed to parse theme file.' };
        }
    }


    #validate(theme: any): theme is Theme {
        if (!theme || typeof theme !== 'object') return false;
        if (typeof theme.id !== 'string' || typeof theme.name !== 'string') return false;
        if (!theme.colors || typeof theme.colors !== 'object') return false;

        const required = ['background', 'primary', 'secondary', 'accent', 'text', 'border'];
        for (const key of required) {
            if (typeof theme.colors[key] !== 'string') return false;
        }

        return true;
    }

    get selectedId() { return this.#selectedId; }
}

export const theme = new ThemeManager();