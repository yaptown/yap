import { useState } from "react"
import { Copy, Check } from "lucide-react"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

interface ErrorMessageProps {
  message: string
  variant?: "default" | "compact"
  className?: string
}

export function ErrorMessage({ message, variant = "default", className }: ErrorMessageProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard
      .writeText(message)
      .then(() => {
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      })
      .catch((err) => {
        console.error("Failed to copy error:", err)
      })
  }

  return (
    <div
      className={cn(
        "p-3 bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-800 rounded-lg overflow-hidden",
        className
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <p
          className={cn(
            "text-sm text-red-600 dark:text-red-400 flex-1 overflow-hidden",
            variant === "compact" ? "line-clamp-3" : "line-clamp-5"
          )}
        >
          {message}
        </p>
        <Button
          variant="ghost"
          size="sm"
          className="h-6 w-6 p-0 flex-shrink-0"
          onClick={handleCopy}
          title={copied ? "Copied!" : "Copy full error message"}
        >
          {copied ? (
            <Check className="h-3 w-3 text-green-600 dark:text-green-400" />
          ) : (
            <Copy className="h-3 w-3" />
          )}
        </Button>
      </div>
    </div>
  )
}
