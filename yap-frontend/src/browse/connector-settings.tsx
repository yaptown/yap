import { ArrowUpRight } from "lucide-react";
import { Link } from "react-router-dom";
import { connector_copy } from "../../../yap-frontend-rs/pkg";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { CopyField } from "@/components/CopyField";

/** How to connect an AI assistant to the account through yap's MCP connector. */
export function ConnectorSettings() {
  const copy = connector_copy();
  return (
    <Card>
      <CardHeader>
        <CardTitle>{copy.title}</CardTitle>
        <CardDescription>{copy.body}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3 text-sm text-muted-foreground">
        <CopyField label={copy.url_label} value={copy.url} />
        <p>{copy.claude_step}</p>
        <p>{copy.chatgpt_step}</p>
        <Link
          to={new URL(copy.guide_url).pathname}
          className="inline-flex items-center gap-1 self-start text-foreground underline"
        >
          {copy.guide_action}
          <ArrowUpRight className="size-4" aria-hidden />
        </Link>
      </CardContent>
    </Card>
  );
}
