import { useEffect, useState } from "react";
import { Trash2 } from "lucide-react";
import { api, errMessage } from "@/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn, formatCurrency } from "@/lib/utils";
import type { PaymentEvent } from "@/types";

type Props = {
  entityId: string;
  documentId: string;
  status?: string | null;
  amountDue?: number | null;
  amountPaid?: number | null;
  compact?: boolean;
  onChanged?: () => void;
  onError?: (message: string) => void;
};

export function PaymentStatusControls({
  entityId,
  documentId,
  status,
  amountDue,
  amountPaid,
  compact = false,
  onChanged,
  onError,
}: Props) {
  const [busy, setBusy] = useState(false);
  const [partialOpen, setPartialOpen] = useState(false);
  const [events, setEvents] = useState<PaymentEvent[]>([]);
  const [amount, setAmount] = useState("");
  const current = status ?? "pending";

  async function loadEvents() {
    try {
      setEvents(await api.listPaymentEvents(entityId, documentId));
    } catch (err) {
      onError?.(errMessage(err));
    }
  }

  useEffect(() => {
    if (!partialOpen) return;
    void loadEvents();
  }, [partialOpen, entityId, documentId]);

  function confirmLeavePaid(actionLabel: string) {
    if (current !== "paid") return true;
    return confirm(
      `Esta factura está marcada como Pagado. ¿${actionLabel}? Se puede revertir después.`,
    );
  }

  async function setStatus(next: "pending" | "paid") {
    if (next === "pending" && !confirmLeavePaid("cambiarla a Pendiente")) return;
    if (current === "paid" && next === "paid") return;
    setBusy(true);
    try {
      await api.updatePayment({
        entityId,
        documentId,
        status: next,
        paidAt: next === "paid" ? new Date().toISOString().slice(0, 10) : null,
      });
      setPartialOpen(false);
      onChanged?.();
    } catch (err) {
      onError?.(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function openPartial() {
    if (!confirmLeavePaid("pasar a pagos parciales")) return;
    setPartialOpen(true);
    await loadEvents();
  }

  async function addPartial() {
    const value = Number(String(amount).replace(",", "."));
    if (!Number.isFinite(value) || value <= 0) {
      onError?.("Ingresá un monto válido mayor a 0");
      return;
    }
    setBusy(true);
    try {
      await api.addPaymentPartial({
        entityId,
        documentId,
        amount: value,
        paidAt: new Date().toISOString().slice(0, 10),
      });
      setAmount("");
      await loadEvents();
      onChanged?.();
    } catch (err) {
      onError?.(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function removePartial(eventId: string) {
    if (!confirm("¿Eliminar este pago parcial?")) return;
    setBusy(true);
    try {
      await api.deletePaymentEvent(entityId, documentId, eventId);
      await loadEvents();
      onChanged?.();
    } catch (err) {
      onError?.(errMessage(err));
    } finally {
      setBusy(false);
    }
  }

  const paidSum =
    events.length > 0 ? events.reduce((sum, ev) => sum + ev.amount, 0) : (amountPaid ?? 0);

  return (
    <div
      className={cn("flex flex-col gap-2", compact ? "min-w-[11rem]" : "")}
      onClick={(e) => e.stopPropagation()}
    >
      <div className="flex flex-wrap gap-1">
        <Button
          type="button"
          size="sm"
          disabled={busy}
          variant={current === "pending" ? "secondary" : "outline"}
          className={cn(current === "pending" && "ring-2 ring-amber-500/40")}
          onClick={() => void setStatus("pending")}
        >
          Pendiente
        </Button>
        <Button
          type="button"
          size="sm"
          disabled={busy}
          variant={current === "partial" ? "default" : "outline"}
          className={cn(current === "partial" && "ring-2 ring-brand-blue/40")}
          onClick={() => void openPartial()}
        >
          Parcial
        </Button>
        <Button
          type="button"
          size="sm"
          disabled={busy}
          variant={current === "paid" ? "success" : "outline"}
          className={cn(current === "paid" && "ring-2 ring-brand-green/50")}
          onClick={() => void setStatus("paid")}
        >
          Pagado
        </Button>
      </div>

      {partialOpen ? (
        <div className="rounded-xl border border-border bg-muted/40 p-2 text-xs">
          <div className="mb-1 flex justify-between gap-2 text-muted-foreground">
            <span>Pagado</span>
            <span className="tabular-nums text-foreground">
              {formatCurrency(paidSum)}
              {amountDue != null ? ` / ${formatCurrency(amountDue)}` : ""}
            </span>
          </div>
          {events.length > 0 ? (
            <ul className="mb-2 max-h-28 space-y-1 overflow-auto">
              {events.map((ev) => (
                <li key={ev.id} className="flex items-center justify-between gap-2">
                  <span>{ev.paidAt.slice(0, 10)}</span>
                  <div className="flex items-center gap-1">
                    <span className="tabular-nums font-medium">{formatCurrency(ev.amount)}</span>
                    <Button
                      type="button"
                      size="icon"
                      variant="ghost"
                      className="h-7 w-7 text-danger"
                      title="Eliminar pago parcial"
                      disabled={busy}
                      onClick={() => void removePartial(ev.id)}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mb-2 text-muted-foreground">Sin pagos parciales aún.</p>
          )}
          <div className="flex gap-1">
            <Input
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="Monto"
              inputMode="decimal"
              className="h-8"
            />
            <Button type="button" size="sm" disabled={busy} onClick={() => void addPartial()}>
              Agregar
            </Button>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              disabled={busy}
              onClick={() => setPartialOpen(false)}
            >
              Cerrar
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
