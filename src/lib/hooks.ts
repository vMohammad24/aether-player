import {
    arrow,
    autoUpdate,
    computePosition,
    flip,
    offset,
    shift
} from '@floating-ui/dom';
import { mount, unmount } from 'svelte';
import type { Action } from 'svelte/action';
import TooltipPopup from './components/TooltipPopup.svelte';

interface TrapFocusParams {
    onClose: () => void;
}


interface TooltipParams {
    content: string;
    placement?: 'top' | 'bottom' | 'left' | 'right';
    delay?: number;
    class?: string;
}

export const tooltip: Action<HTMLElement, string | TooltipParams> = (node, params) => {
    let tooltipComponent: ReturnType<typeof mount> | null = null;
    let cleanupFloating: (() => void) | null = null;
    let timeoutId: number | undefined;
    let config = normalizeParams(params);

    function normalizeParams(p: string | TooltipParams): TooltipParams {
        return typeof p === 'string'
            ? { content: p, placement: 'top', delay: 200 }
            : { placement: 'top', delay: 200, ...p };
    }

    function updatePosition() {
        if (!tooltipComponent) return;
        const tooltipEl = document.getElementById('tooltip-portal-root');
        const arrowEl = document.getElementById('tooltip-arrow');

        if (!tooltipEl || !arrowEl) return;

        computePosition(node, tooltipEl, {
            placement: config.placement,
            middleware: [
                offset(8),
                flip(),
                shift({ padding: 5 }),
                arrow({ element: arrowEl })
            ]
        }).then(({ x, y, placement, middlewareData }) => {
            Object.assign(tooltipEl.style, {
                left: `${x}px`,
                top: `${y}px`
            });
            const { x: arrowX, y: arrowY } = middlewareData.arrow || {};
            const staticSide = {
                top: 'bottom',
                right: 'left',
                bottom: 'top',
                left: 'right'
            }[placement.split('-')[0]] || 'bottom';

            Object.assign(arrowEl.style, {
                left: arrowX != null ? `${arrowX}px` : '',
                top: arrowY != null ? `${arrowY}px` : '',
                right: '',
                bottom: '',
                [staticSide]: '-4px'
            });
        });
    }

    function show() {
        if (tooltipComponent) return;

        tooltipComponent = mount(TooltipPopup, {
            target: document.body,
            props: { content: config.content, class: config.class }
        });

        setTimeout(() => {
            const tooltipEl = document.getElementById('tooltip-portal-root');
            const arrowEl = document.getElementById('tooltip-arrow');

            if (tooltipEl && arrowEl) {
                cleanupFloating = autoUpdate(node, tooltipEl, updatePosition);
            }
        }, 0);
    }

    function hide() {
        clearTimeout(timeoutId);
        if (cleanupFloating) cleanupFloating();
        if (tooltipComponent) {
            unmount(tooltipComponent);
            tooltipComponent = null;
        }
    }

    function handleMouseEnter() {
        timeoutId = window.setTimeout(show, config.delay);
    }

    function handleMouseLeave() {
        clearTimeout(timeoutId);
        hide();
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape') hide();
    }

    node.addEventListener('mouseenter', handleMouseEnter);
    node.addEventListener('mouseleave', handleMouseLeave);
    node.addEventListener('focus', handleMouseEnter);
    node.addEventListener('blur', handleMouseLeave);
    window.addEventListener('keydown', handleKeyDown);

    return {
        update(newParams) {
            config = normalizeParams(newParams);
            if (tooltipComponent) {
                hide();
                show();
            }
        },
        destroy() {
            hide();
            node.removeEventListener('mouseenter', handleMouseEnter);
            node.removeEventListener('mouseleave', handleMouseLeave);
            node.removeEventListener('focus', handleMouseEnter);
            node.removeEventListener('blur', handleMouseLeave);
            window.removeEventListener('keydown', handleKeyDown);
        }
    };
};

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