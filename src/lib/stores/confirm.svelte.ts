import ConfirmModal from "$lib/components/ConfirmModal.svelte";
import { mount, unmount } from "svelte";






export const confirm = (message: string, title?: string, variant?: "danger" | "warning" | "info"): Promise<boolean> => {
    return new Promise((resolve) => {
        const modalComponent: ReturnType<typeof mount> = mount(ConfirmModal, {
            target: document.body,
            props: {
                show: true,
                confirmText: 'Confirm',
                declineText: 'Cancel',
                message,
                title,
                variant,
                onConfirm: () => finish(true),
                onDecline: () => finish(false),
                onclose: () => finish(false)
            }
        });
        const finish = (state: boolean) => {
            resolve(state);
            setTimeout(() => {
                unmount(modalComponent);
            }, 3000);
        }
    })
}