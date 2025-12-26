import type { Component } from 'svelte';

export type ToastType = 'info' | 'success' | 'warning' | 'error' | 'loading';
export type ToastPosition = 'top-left' | 'top-center' | 'top-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';

export interface ToastAction {
    label: string;
    onClick: () => void;
    variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive';
    size?: 'sm' | 'md' | 'lg';
}

export interface Toast {
    id: string;
    type: ToastType;
    message: string;
    title?: string;
    duration?: number;
    dismissible?: boolean;
    icon?: Component;
    actions?: ToastAction[];

    createdAt: number;
    paused?: boolean;
}

class Timer {
    private timerId?: number;
    private start: number;
    private remaining: number;
    private callback: () => void;

    constructor(callback: () => void, delay: number) {
        this.remaining = delay;
        this.callback = callback;
        this.start = Date.now();
        this.resume();
    }

    pause() {
        if (this.timerId) {
            clearTimeout(this.timerId);
            this.timerId = undefined;
            this.remaining -= Date.now() - this.start;
        }
    }

    resume() {
        if (this.timerId) return;
        if (this.remaining <= 0) {
            this.callback();
            return;
        }
        this.start = Date.now();
        this.timerId = window.setTimeout(this.callback, this.remaining);
    }

    clear() {
        if (this.timerId) clearTimeout(this.timerId);
    }
}

class ToastStore {
    toasts = $state<Toast[]>([]);
    private timers = new Map<string, Timer>();
    private readonly MAX_TOASTS = 5;

    add(data: Partial<Toast> & { message: string }) {
        const id = crypto.randomUUID();
        const { duration = 5000, type = 'info', ...rest } = data;

        const newToast: Toast = {
            id,
            type,
            duration,
            dismissible: true,
            createdAt: Date.now(),
            ...rest
        };


        if (this.toasts.length >= this.MAX_TOASTS) {
            const oldest = this.toasts[this.toasts.length - 1];
            this.remove(oldest.id);
        }


        this.toasts = [newToast, ...this.toasts];


        if (duration > 0 && type !== 'loading') {
            this.timers.set(id, new Timer(() => this.remove(id), duration));
        }

        return id;
    }


    update(id: string, data: Partial<Toast>) {
        const index = this.toasts.findIndex((t) => t.id === id);
        if (index === -1) return;


        if (this.timers.has(id)) {
            this.timers.get(id)?.clear();
            this.timers.delete(id);
        }


        const updatedToast = { ...this.toasts[index], ...data };
        this.toasts[index] = updatedToast;


        if (updatedToast.duration && updatedToast.duration > 0 && updatedToast.type !== 'loading') {
            this.timers.set(id, new Timer(() => this.remove(id), updatedToast.duration));
        }
    }

    remove(id: string) {
        const timer = this.timers.get(id);
        if (timer) {
            timer.clear();
            this.timers.delete(id);
        }
        this.toasts = this.toasts.filter((t) => t.id !== id);
    }


    pause(id: string) {
        this.timers.get(id)?.pause();
    }

    resume(id: string) {
        this.timers.get(id)?.resume();
    }
}

export const toastState = new ToastStore();

export const toast = {
    message: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, ...opts }),
    info: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, type: 'info', ...opts }),
    success: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, type: 'success', ...opts }),
    warning: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, type: 'warning', ...opts }),
    error: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, type: 'error', ...opts }),
    loading: (msg: string, opts?: Partial<Toast>) => toastState.add({ message: msg, type: 'loading', duration: 0, ...opts }),
    dismiss: (id: string) => toastState.remove(id),

    promise: <T>(
        promise: Promise<T>,
        msgs: { loading: string; success: string | ((data: T) => string); error: string | ((err: unknown) => string) },
        opts?: Partial<Toast>
    ) => {
        const id = toastState.add({ message: msgs.loading, type: 'loading', duration: 0, ...opts });
        promise
            .then((data) => {
                const message = typeof msgs.success === 'function' ? msgs.success(data) : msgs.success;
                toastState.update(id, { type: 'success', message, duration: 4000 });
            })
            .catch((err) => {
                const message = typeof msgs.error === 'function' ? msgs.error(err) : msgs.error;
                toastState.update(id, { type: 'error', message, duration: 5000 });
            });
        return promise;
    }
};