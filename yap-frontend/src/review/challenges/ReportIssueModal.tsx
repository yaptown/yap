import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { supabase } from "@/lib/supabase";
import {
  report_issue,
  report_issue_copy,
  type IssueSubject,
  type Language,
} from "../../../../yap-frontend-rs/pkg";

interface ReportIssueModalProps {
  subject: IssueSubject;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  targetLanguage: Language;
}

export function ReportIssueModal({
  subject,
  open,
  onOpenChange,
  targetLanguage,
}: ReportIssueModalProps) {
  const [issueText, setIssueText] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [failed, setFailed] = useState(false);
  const copy = report_issue_copy();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!issueText.trim()) {
      return;
    }

    setIsSubmitting(true);
    setFailed(false);

    try {
      const {
        data: { session },
      } = await supabase.auth.getSession();

      if (!session) {
        throw new Error("Must be logged in to report issues");
      }

      await report_issue(
        targetLanguage,
        subject,
        issueText,
        session.user.id,
        session.access_token,
      );

      setIssueText("");
      onOpenChange(false);
    } catch (error) {
      console.error("Failed to submit issue:", error);
      setFailed(true);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{copy.title}</DialogTitle>
          <DialogDescription>{copy.description}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="issue-text">{copy.field_label}</Label>
              <Textarea
                id="issue-text"
                placeholder={copy.placeholder}
                value={issueText}
                onChange={(e) => setIssueText(e.target.value)}
                className="min-h-[100px]"
                required
              />
              {failed && (
                <p className="text-sm text-destructive">{copy.failed_label}</p>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isSubmitting}
            >
              {copy.cancel_label}
            </Button>
            <Button type="submit" disabled={isSubmitting || !issueText.trim()}>
              {isSubmitting ? copy.submitting_label : copy.submit_label}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
