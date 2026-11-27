import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatCurrency(value: number) {
  return new Intl.NumberFormat("es-AR", {
    style: "currency",
    currency: "ARS",
    maximumFractionDigits: 0,
  }).format(value);
}

export function paymentStatusLabel(status?: string | null) {
  switch (status) {
    case "paid":
      return "Pagado";
    case "partial":
      return "Parcial";
    case "pending":
      return "Pendiente";
    default:
      return status || "—";
  }
}

/** Rojo vencida · amarillo ≤7 días · verde >7 días · neutro si pagado o sin fecha. */
export function dueDateTone(
  dueDate?: string | null,
  status?: string | null,
): "overdue" | "soon" | "ok" | "neutral" {
  if (!dueDate || status === "paid") return "neutral";
  const due = Date.parse(`${dueDate.slice(0, 10)}T00:00:00`);
  if (Number.isNaN(due)) return "neutral";
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const diffDays = Math.round((due - today.getTime()) / 86_400_000);
  if (diffDays < 0) return "overdue";
  if (diffDays <= 7) return "soon";
  return "ok";
}

export function dueDateClass(dueDate?: string | null, status?: string | null) {
  switch (dueDateTone(dueDate, status)) {
    case "overdue":
      return "font-medium text-red-600 dark:text-red-400";
    case "soon":
      return "font-medium text-amber-600 dark:text-amber-400";
    case "ok":
      return "font-medium text-emerald-600 dark:text-emerald-400";
    default:
      return "text-muted-foreground";
  }
}
