import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";
import { TableHead } from "@/components/ui/table";
import { cn } from "@/lib/utils";

type Props = {
  label: string;
  active: boolean;
  dir: "asc" | "desc";
  onClick: () => void;
  className?: string;
};

export function SortableHead({ label, active, dir, onClick, className }: Props) {
  const Icon = !active ? ArrowUpDown : dir === "asc" ? ArrowUp : ArrowDown;
  const right = className?.includes("text-right");
  return (
    <TableHead className={className}>
      <button
        type="button"
        className={cn(
          "inline-flex items-center gap-1 font-medium hover:text-foreground",
          right ? "w-full justify-end text-right" : "text-left",
          active ? "text-foreground" : "text-muted-foreground",
        )}
        onClick={onClick}
      >
        {label}
        <Icon className="h-3.5 w-3.5 opacity-70" />
      </button>
    </TableHead>
  );
}

export function toggleSortState<K extends string>(
  currentKey: K,
  currentDir: "asc" | "desc",
  nextKey: K,
): { key: K; dir: "asc" | "desc" } {
  if (currentKey === nextKey) {
    return { key: nextKey, dir: currentDir === "asc" ? "desc" : "asc" };
  }
  return { key: nextKey, dir: "asc" };
}
