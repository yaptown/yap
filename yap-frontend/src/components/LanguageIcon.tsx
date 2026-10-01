import { cn } from "@/lib/pure";

const assets = import.meta.glob<string>("../assets/language-icons/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});

/** Decorative course artwork; the adjacent language name provides its label. */
export function LanguageIcon({
  icon,
  variant = "tile",
  className,
}: {
  icon: string;
  variant?: "tile" | "bare";
  className?: string;
}) {
  const src = (suffix: string) => assets[`../assets/language-icons/${icon}${suffix}.svg`];
  if (variant === "tile") {
    return <img src={src("")} alt="" aria-hidden className={cn("shrink-0", className)} />;
  }
  return (
    <>
      <img src={src("-bare")} alt="" aria-hidden className={cn("shrink-0", className, "inline-block dark:hidden")} />
      <img src={src("-bare-dark")} alt="" aria-hidden className={cn("shrink-0", className, "hidden dark:inline-block")} />
    </>
  );
}
