// A note from the person who made Yap, read while the deck builds. The
// portrait is cropped out of the about page's selfie rather than shipped as
// its own image.
export function Backstory({ text, signature }: { text: string; signature: string }) {
  return (
    <figure className="animate-fade-in flex flex-col gap-4 rounded-xl border bg-card/60 p-5 backdrop-blur-sm">
      <blockquote className="text-pretty leading-relaxed">{text}</blockquote>
      <figcaption className="flex items-center gap-3 text-sm text-muted-foreground">
        <span
          aria-hidden
          className="size-9 shrink-0 rounded-full border bg-no-repeat"
          style={{ backgroundImage: "url(/selfie.webp)", backgroundSize: "380% auto", backgroundPosition: "40.5% 21%" }}
        />
        {signature}
      </figcaption>
    </figure>
  );
}
