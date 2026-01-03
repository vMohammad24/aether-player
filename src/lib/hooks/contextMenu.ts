import { mount, unmount, type Component } from 'svelte';
import type { Action } from 'svelte/action';
import ContextMenuPopup from '../components/popups/ContextMenuPopup.svelte';

interface BaseItem {
    label: string;
    disabled?: boolean;
    shortcut?: string;
    leftIcon?: Component;
    rightIcon?: Component;
}

export type ContextMenuItem =
    | { type: 'separator' }
    | { type: 'label'; label: string }
    | ({ type?: 'action'; onClick?: () => void; variant?: 'default' | 'danger'; children?: ContextMenuItem[] } & BaseItem)
    | ({ type: 'checkbox'; checked: boolean; onChange?: (checked: boolean) => void } & BaseItem)
    | ({ type: 'radio'; checked: boolean; onSelect?: () => void } & BaseItem);

let activeMenu: { unmount: () => void; element: HTMLElement } | null = null;

export const contextMenu: Action<HTMLElement, ContextMenuItem[]> = (node, items) => {
    let currentItems = items;

    function close() {
        if (activeMenu) {
            activeMenu.unmount();
            activeMenu = null;
        }
        window.removeEventListener('click', close);
        window.removeEventListener('contextmenu', handleOutsideClick);
        window.removeEventListener('resize', close);
        window.removeEventListener('scroll', close, true);
        window.removeEventListener('keydown', handleGlobalKeydown);
    }

    function handleGlobalKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') close();
    }

    function handleOutsideClick(e: MouseEvent) {
        if (activeMenu?.element.contains(e.target as Node)) return;
        close();
    }

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        close();


        const virtualElement = {
            getBoundingClientRect: () => ({
                width: 0, height: 0, x: e.clientX, y: e.clientY,
                top: e.clientY, left: e.clientX, right: e.clientX, bottom: e.clientY,
            }),
            contextElement: node
        };

        const component = mount(ContextMenuPopup, {
            target: document.body,
            props: {
                items: currentItems,
                anchor: virtualElement,
                onClose: close,
                placement: 'bottom-start'
            },
        });

        activeMenu = {
            unmount: () => unmount(component),
            element: document.body
        };


        setTimeout(() => {
            window.addEventListener('click', close);
            window.addEventListener('contextmenu', handleOutsideClick);
            window.addEventListener('resize', close);
            window.addEventListener('scroll', close, true);
            window.addEventListener('keydown', handleGlobalKeydown);
        }, 0);
    }

    node.addEventListener('contextmenu', handleContextMenu);

    return {
        update(newItems) { currentItems = newItems; },
        destroy() {
            node.removeEventListener('contextmenu', handleContextMenu);
            if (activeMenu) close();
        },
    };
};