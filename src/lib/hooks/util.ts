import type { Action } from 'svelte/action';



interface TrapFocusParams {
    onClose: () => void;
}

export const clickOutside: Action<HTMLElement, () => void> = (node, callback) => {
    function handleClick(event: MouseEvent) {
        const target = event.target as Node;
        if (!node || node.contains(target) || !document.contains(target)) {
            return;
        }

        callback();
    }

    document.addEventListener('click', handleClick, true);

    return {
        destroy() {
            document.removeEventListener('click', handleClick, true);
        }
    };
};


export const trapFocus: Action<HTMLElement, TrapFocusParams> = (node, { onClose }) => {
    const previousActiveElement = document.activeElement as HTMLElement;

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            e.stopPropagation();
            onClose();
            return;
        }

        if (e.key === 'Tab') {
            const selector = 'a[href], button, input, textarea, select, details, [tabindex]:not([tabindex="-1"])';
            const focusable = Array.from(node.querySelectorAll<HTMLElement>(selector))
                .filter(el => !el.hasAttribute('disabled') && el.getAttribute('aria-hidden') !== 'true');

            const first = focusable[0];
            const last = focusable[focusable.length - 1];

            if (e.shiftKey && document.activeElement === first) {
                e.preventDefault();
                last?.focus();
            } else if (!e.shiftKey && document.activeElement === last) {
                e.preventDefault();
                first?.focus();
            }
        }
    }

    const startFocus = node.querySelector<HTMLElement>('[autofocus]') || node;
    startFocus.focus();

    window.addEventListener('keydown', handleKeydown);

    return {
        update(newParams) {
            onClose = newParams.onClose;
        },
        destroy() {
            window.removeEventListener('keydown', handleKeydown);
            previousActiveElement?.focus();
        }
    };
};

export const lockScroll: Action = (_node) => {
    const originalOverflow = document.body.style.overflow;
    const originalPadding = document.body.style.paddingRight;

    const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;

    document.body.style.paddingRight = `${scrollbarWidth}px`;
    document.body.style.overflow = 'hidden';

    return {
        destroy() {
            document.body.style.overflow = originalOverflow;
            document.body.style.paddingRight = originalPadding;
        }
    };
};