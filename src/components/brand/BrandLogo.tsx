import folioIcon from "@/assets/brand/folio_icon.svg";
import { cn } from "@/lib/utils";

type BrandLogoProps = {
  variant?: "icon" | "wordmark" | "full";
  className?: string;
  alt?: string;
};

/** Wordmark en HTML: el SVG con <text> falla como <img> porque no embebe la fuente. */
function Wordmark({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "relative inline-block font-sans text-[1.65rem] font-bold leading-none tracking-[-0.05em] text-brand-navy",
        className,
      )}
      aria-hidden="true"
    >
      Fol
      <span className="relative inline-block">
        i
        <span className="absolute left-1/2 top-[0.08em] h-[0.26em] w-[0.26em] -translate-x-1/2 rounded-full bg-brand-green" />
      </span>
      o
    </span>
  );
}

export function BrandLogo({
  variant = "icon",
  className,
  alt = "Folio",
}: BrandLogoProps) {
  if (variant === "wordmark") {
    return (
      <span className={cn("inline-flex items-center", className)} role="img" aria-label={alt}>
        <Wordmark />
      </span>
    );
  }

  if (variant === "full") {
    return (
      <span
        className={cn("inline-flex items-center gap-3", className)}
        role="img"
        aria-label={alt}
      >
        <img src={folioIcon} alt="" className="h-12 w-12 shrink-0" />
        <Wordmark className="text-[2.1rem]" />
      </span>
    );
  }

  return (
    <img
      src={folioIcon}
      alt={alt}
      className={cn("h-9 w-9", className)}
    />
  );
}
