import { useState } from "react";
import {
  account_copy,
  is_pending_review_key_for_user,
} from "../../../yap-frontend-rs/pkg";
import { supabase } from "@/lib/supabase";
import { useWeapon } from "@/core/weapon";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Trash2 } from "lucide-react";

export function DeleteAccountSettings() {
  const weapon = useWeapon();
  const copy = account_copy();
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);

  const handleDelete = async () => {
    setBusy(true);
    setFailed(false);
    try {
      const { data } = await supabase.auth.getSession();
      await weapon.delete_account(data.session?.access_token ?? "");
      // Unfinished review drafts live in localStorage, outside the Weapon.
      const userId = data.session?.user.id ?? "";
      Object.keys(localStorage)
        .filter((key) => is_pending_review_key_for_user(key, userId))
        .forEach((key) => localStorage.removeItem(key));
      // The user is gone server-side, so only the local session is left to drop.
      await supabase.auth.signOut({ scope: "local" });
    } catch (error) {
      console.error("Account deletion failed", error);
      setFailed(true);
      setBusy(false);
    }
  };

  return (
    <Dialog onOpenChange={() => setFailed(false)}>
      <DialogTrigger asChild>
        <DropdownMenuItem
          variant="destructive"
          onSelect={(e) => e.preventDefault()}
        >
          <Trash2 className="mr-2 h-4 w-4" />
          {copy.delete_account_action}
        </DropdownMenuItem>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{copy.delete_account_title}</DialogTitle>
          <DialogDescription>{copy.delete_account_body}</DialogDescription>
        </DialogHeader>
        {failed && (
          <p className="text-sm text-destructive">
            {copy.delete_account_failed}
          </p>
        )}
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" disabled={busy}>
              Cancel
            </Button>
          </DialogClose>
          <Button variant="destructive" onClick={handleDelete} disabled={busy}>
            {busy ? copy.deleting_account : copy.delete_account_confirm}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
