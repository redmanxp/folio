import { AlertCircle, CheckCircle2, Info, TriangleAlert } from "lucide-react";
import type { HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

const styles = {
  info: {
    className: "border-brand-blue/20 bg-brand-blue/5 text-foreground",
    icon: Info,
  },
  success: {
    className: "border-brand-green/20 bg-brand-green/5 text-foreground",
    icon: CheckCircle2,
  },
  warning: {
    className: "border-warning/20 bg-warning/5 text-foreground",
    icon: TriangleAlert,
  },
  danger: {
    className: "border-danger/20 bg-danger/5 text-foreground",
    icon: AlertCircle,
  },
} as const;

type AlertProps = HTMLAttributes<HTMLDivElement> & {
  variant?: keyof typeof styles;
  title?: string;
};

export function Alert({ variant = "info", title, className, children, ...props }: AlertProps) {
  const config = styles[variant];
  const Icon = config.icon;

  return (
    <div
      className={cn(
        "flex gap-3 rounded-2xl border px-4 py-3 text-sm",
        config.className,
        className,
      )}
      {...props}
    >
      <Icon className="mt-0.5 h-4 w-4 shrink-0 text-brand-blue" />
      <div>
        {title ? <p className="font-medium">{title}</p> : null}
        <div className={cn(title ? "mt-1 text-muted-foreground" : "")}>{children}</div>
      </div>
    </div>
  );
}
