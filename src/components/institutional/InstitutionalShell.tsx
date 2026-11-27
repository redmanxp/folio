import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

export function InstitutionalShell({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cn("mx-auto flex max-w-3xl flex-col gap-4", className)}>{children}</div>;
}

export function InfoCard({
  title,
  description,
  children,
  action,
}: {
  title: string;
  description?: string;
  children?: ReactNode;
  action?: ReactNode;
}) {
  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle>{title}</CardTitle>
          {description ? <CardDescription>{description}</CardDescription> : null}
        </div>
        {action}
      </CardHeader>
      {children ? <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">{children}</CardContent> : null}
    </Card>
  );
}

export function FeatureGrid({
  items,
}: {
  items: { title: string; body: string }[];
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-2">
      {items.map((item) => (
        <div key={item.title} className="rounded-2xl border border-border bg-muted/30 p-4">
          <p className="text-sm font-semibold text-foreground">{item.title}</p>
          <p className="mt-1 text-sm text-muted-foreground">{item.body}</p>
        </div>
      ))}
    </div>
  );
}

export function SoonButton({ label }: { label: string }) {
  return (
    <Button type="button" variant="outline" disabled title="Se habilitará cuando estén los enlaces">
      {label} · próximamente
    </Button>
  );
}
