import { t } from "@/lib/i18n";
import { useState, useEffect } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { supabase } from "@/lib/supabase";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ThemeProvider } from "@/components/theme-provider";

export function AcceptInvite() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [email, setEmail] = useState("");

  const token_hash = searchParams.get("token_hash");
  const type = searchParams.get("type");

  useEffect(() => {
    if (token_hash && type) {
      // Extract email from URL if available
      const urlEmail = searchParams.get("email");
      if (urlEmail) {
        setEmail(decodeURIComponent(urlEmail));
      }
    } else {
      setError(t("accept-invite-invalid-invite-link"));
    }
  }, [token_hash, type, searchParams]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (password !== confirmPassword) {
      setError(t("accept-invite-passwords-do-not-match"));
      return;
    }

    if (password.length < 6) {
      setError(t("accept-invite-password-must-be-at-least-6-characters"));
      return;
    }

    setLoading(true);

    if (token_hash && type) {
      const { error } = await supabase.auth.verifyOtp({
        token_hash,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        type: type as any,
      });

      if (error) {
        setError(error.message);
        setLoading(false);
        return;
      }

      const { error: updateError } = await supabase.auth.updateUser({
        password: password,
      });

      if (updateError) {
        setError(updateError.message);
      } else {
        setSuccess(true);
        setTimeout(() => {
          navigate("/");
        }, 2000);
      }
    } else {
      setError(t("accept-invite-invalid-invite-link"));
    }

    setLoading(false);
  };

  if (success) {
    return (
      <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl font-bold text-positive">
                {t("accept-invite-welcome-to-yap-town")}
              </CardTitle>
              <CardDescription>
                {t(
                  "accept-invite-your-account-has-been-set-up-successfully-redirecting",
                )}
              </CardDescription>
            </CardHeader>
          </Card>
        </div>
      </ThemeProvider>
    );
  }

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="min-h-screen bg-background flex items-center justify-center p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <CardTitle className="text-2xl font-bold">
              {t("accept-invite-accept-invitation")}
            </CardTitle>
            <CardDescription>
              {email
                ? t("auth-invite-password-for", {
                    email: String(email),
                  })
                : t(
                    "accept-invite-set-up-your-password-to-complete-registration",
                  )}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              {email && (
                <div className="space-y-2">
                  <Label>{t("auth-email")}</Label>
                  <Input
                    type="email"
                    value={email}
                    disabled
                    className="bg-muted"
                  />
                </div>
              )}
              <div className="space-y-2">
                <Label htmlFor="password">{t("auth-password")}</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  minLength={6}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="confirm-password">
                  {t("accept-invite-confirm-password")}
                </Label>
                <Input
                  id="confirm-password"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  required
                  minLength={6}
                />
              </div>
              <Button type="submit" className="w-full" disabled={loading}>
                {loading
                  ? t("accept-invite-setting-up-account")
                  : t("accept-invite-complete-setup")}
              </Button>
            </form>

            {error && (
              <div className="mt-4 p-3 rounded text-sm bg-destructive/10 text-destructive border border-destructive/20">
                {error}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </ThemeProvider>
  );
}
