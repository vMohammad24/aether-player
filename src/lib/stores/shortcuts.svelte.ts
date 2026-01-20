import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { player } from "$lib/stores/player/player.svelte";
import { queue } from "$lib/stores/player/queue.svelte";

export type ShortcutAction =
    | "player.toggle"
    | "player.next"
    | "player.prev"
    | "player.repeat"
    | "player.volumeup"
    | "player.volumedown"
    | "player.mute"
    | "app.search"
    | "app.settings"
    | "sidebar.toggle"
    | "sidebar.scroll";

export type KeyCombo = {
    key: string;
    ctrl?: boolean;
    shift?: boolean;
    alt?: boolean;
};

export type ShortcutConfig = Record<ShortcutAction, KeyCombo>;
const DEFAULT_BINDINGS: ShortcutConfig = {
    "player.toggle": { key: " " },
    "player.next": { key: "ArrowRight", ctrl: true },
    "player.prev": { key: "ArrowLeft", ctrl: true },
    "player.repeat": { key: "r", ctrl: true },
    "app.search": { key: "k", ctrl: true },
    "app.settings": { key: ",", ctrl: true },
    "player.volumeup": { key: "ArrowUp", ctrl: true },
    "player.volumedown": { key: "ArrowDown", ctrl: true },
    "player.mute": { key: "m", ctrl: true },
    "sidebar.toggle": { key: "b", ctrl: true },
    "sidebar.scroll": { key: "Tab", ctrl: true },
};

class ShortcutManager {
    #bindings = $state<ShortcutConfig>(DEFAULT_BINDINGS);
    #callbacks = new Map<ShortcutAction, () => void>();
    #listening = false;
    #boundHandler = (event: KeyboardEvent) => this.handleKeydown(event);


    on(action: ShortcutAction, callback: () => void) {
        this.#callbacks.set(action, callback);
        return () => this.#callbacks.delete(action);
    }


    handleKeydown(event: KeyboardEvent) {
        if (event.defaultPrevented || event.isComposing || event.repeat) return;
        if (!browser) return;


        const target = this.#getEventTarget(event);
        const isInput = this.#isEditableTarget(target);


        const hasModifier = event.ctrlKey || event.metaKey || event.altKey;
        if (isInput && !hasModifier) return;


        for (const [action, combo] of Object.entries(this.#bindings)) {
            if (this.#isMatch(event, combo)) {
                event.preventDefault();
                this.#trigger(action as ShortcutAction);
                return;
            }
        }
    }

    #isMatch(event: KeyboardEvent, combo: KeyCombo): boolean {
        const keyMatch = this.#normalizeKey(event.key) === this.#normalizeKey(combo.key);

        const modMatch = !!combo.ctrl === (event.ctrlKey || event.metaKey);
        const shiftMatch = !!combo.shift === event.shiftKey;
        const altMatch = !!combo.alt === event.altKey;

        return keyMatch && modMatch && shiftMatch && altMatch;
    }

    #trigger(action: ShortcutAction) {

        if (this.#callbacks.has(action)) {
            this.#callbacks.get(action)?.();
            return;
        }


        switch (action) {
            case "player.toggle": {
                const isPaused = player.state?.paused ?? true;
                if (isPaused) {
                    player.play.trigger();
                } else {
                    player.pause.trigger();
                }
                break;
            }
            case "player.next":
                player.next.trigger();
                break;
            case "player.prev":
                player.prev.trigger();
                break;
            case "player.repeat": {
                const current = queue.data?.repeat ?? "off";
                const next = current === "off" ? "all" : current === "all" ? "one" : "off";
                queue.setRepeat.trigger(next);
                break;
            }
            case "player.volumeup":
                player.setVolume(player.state?.volume! + 0.05);
                break;
            case "player.volumedown":
                player.setVolume(player.state?.volume! - 0.05);
                break;
            case "player.mute":
                player.muted ? player.unmute() : player.mute();
                break;
            case "app.settings":
                goto("/settings");
                break;
            default:
                break;
        }
    }

    start() {
        if (!browser || this.#listening) return;
        window.addEventListener("keydown", this.#boundHandler);
        this.#listening = true;
    }

    stop() {
        if (!browser || !this.#listening) return;
        window.removeEventListener("keydown", this.#boundHandler);
        this.#listening = false;
    }

    get bindings() {
        return this.#bindings;
    }

    setBindings(next: ShortcutConfig) {
        this.#bindings = next;
    }

    setBinding(action: ShortcutAction, combo: KeyCombo) {
        this.#bindings = { ...this.#bindings, [action]: combo };
    }

    resetBindings() {
        this.#bindings = DEFAULT_BINDINGS;
    }

    #normalizeKey(key: string) {
        if (key === " ") return "space";
        if (key.toLowerCase() === "spacebar") return "space";
        return key.toLowerCase();
    }

    #getEventTarget(event: KeyboardEvent) {
        const path = event.composedPath?.();
        if (path && path.length > 0) {
            const el = path[0];
            if (el instanceof HTMLElement) return el;
        }
        return event.target instanceof HTMLElement ? event.target : null;
    }

    #isEditableTarget(target: HTMLElement | null) {
        if (!target) return false;
        const tag = target.tagName;
        return tag === "INPUT" || tag === "TEXTAREA" || target.isContentEditable;
    }
}

export const shortcuts = new ShortcutManager();