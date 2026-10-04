import { cn } from "@/lib/pure";

// Both directories come from the yap-icons repo, in one house style: course
// icons, and illustrations for the app's own screens (ids "ui.*").
const assets = Object.fromEntries(
  Object.entries(
    import.meta.glob<string>(["../assets/language-icons/*.svg", "../assets/illustrations/*.svg"], {
      eager: true,
      query: "?url",
      import: "default",
    }),
  ).map(([path, url]) => [path.slice(path.lastIndexOf("/") + 1), url]),
);

/** Decorative artwork; adjacent text provides its label. */
export function LanguageIcon({
  icon,
  variant = "tile",
  className,
}: {
  icon: string;
  variant?: "tile" | "bare";
  className?: string;
}) {
  const src = (suffix: string) => assets[`${icon}${suffix}.svg`];
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
