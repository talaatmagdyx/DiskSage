import { create } from "zustand";

export type ToastTone = "success" | "info" | "warning" | "error";

export type ToastMessage = {
  id: string;
  title: string;
  message?: string;
  tone: ToastTone;
};

type ToastState = {
  messages: ToastMessage[];
  add: (toast: Omit<ToastMessage, "id">) => string;
  dismiss: (id: string) => void;
};

export const useToastStore = create<ToastState>((set) => ({
  messages: [],
  add: (toast) => {
    const id = crypto.randomUUID();
    set((state) => ({ messages: [...state.messages.slice(-3), { ...toast, id }] }));
    return id;
  },
  dismiss: (id) => set((state) => ({ messages: state.messages.filter((toast) => toast.id !== id) })),
}));

export const toast = (message: Omit<ToastMessage, "id">) => useToastStore.getState().add(message);
