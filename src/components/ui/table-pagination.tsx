import { Button } from "@/components/ui/button";

type Props = {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
};

export function TablePagination({ page, pageSize, total, onPageChange }: Props) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  if (total <= pageSize) return null;
  const from = total === 0 ? 0 : page * pageSize + 1;
  const to = Math.min(total, (page + 1) * pageSize);

  return (
    <div className="mt-3 flex flex-wrap items-center justify-between gap-2 text-sm text-muted-foreground">
      <span>
        {from}–{to} de {total}
      </span>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={page <= 0}
          onClick={() => onPageChange(page - 1)}
        >
          Anterior
        </Button>
        <span className="tabular-nums">
          {page + 1} / {totalPages}
        </span>
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={page + 1 >= totalPages}
          onClick={() => onPageChange(page + 1)}
        >
          Siguiente
        </Button>
      </div>
    </div>
  );
}

export const DEFAULT_PAGE_SIZE = 25;
