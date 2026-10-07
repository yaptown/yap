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

export function ResetPassword() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [isValidToken, setIsValidToken] = useState<boolean | null>(null);

  useEffect(() => {
    let access_token = searchParams.get("access_token");
    let refresh_token = searchParams.get("refresh_token");

    // If not in query params, check hash fragments
    if (!access_token && window.location.hash) {
      const hashParams = new URLSearchParams(window.location.hash.substring(1));
      access_token = hashParams.get("access_token");
      refresh_token = hashParams.get("refresh_token");
    }

    if (access_token && refresh_token) {
      // Standard Supabase password reset flow - just set the session
      supabase.auth
        .setSession({
          access_token,
          refresh_token,
        })
        .then(({ data, error }) => {
          if (error) {
            console.error("Error setting session:", error);
            setError(
              t(
                "reset-password-invalid-or-expired-reset-link-please-request-a-new",
              ),
            );
            setIsValidToken(false);
          } else {
            console.log("Session set successfully:", data);
            setIsValidToken(true);
            // Clear the URL to hide the tokens
            window.history.replaceState(null, "", window.location.pathname);
          }
        })
        .catch((e) => {
          console.error("Unexpected error setting session:", e);
          setError(
            t(
              "reset-password-invalid-or-expired-reset-link-please-request-a-new",
            ),
          );
          setIsValidToken(false);
        });
    } else {
      console.log("No tokens found in URL");
      setError(t("reset-password-invalid-reset-link-please-request-a-new-one"));
      setIsValidToken(false);
    }
  }, [searchParams]);

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

    const { error } = await supabase.auth.updateUser({
      password: password,
    });

    if (error) {
      setError(error.message);
    } else {
      setSuccess(true);
      setTimeout(() => {
        navigate("/");
      }, 2000);
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
                {t("reset-password-password-updated")}
              </CardTitle>
              <CardDescription>
                {t(
                  "reset-password-your-password-has-been-successfully-updated-redirecting",
                )}
              </CardDescription>
            </CardHeader>
          </Card>
        </div>
      </ThemeProvider>
    );
  }

  // Show loading while verifying token
  if (isValidToken === null) {
    return (
      <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl font-bold">
                {t("reset-password-verifying-reset-link")}
              </CardTitle>
              <CardDescription>
                {t("reset-password-please-wait")}
              </CardDescription>
            </CardHeader>
          </Card>
        </div>
      </ThemeProvider>
    );
  }

  // Show error if token is invalid
  if (isValidToken === false) {
    return (
      <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl font-bold">
                {t("reset-password-invalid-reset-link")}
              </CardTitle>
              <CardDescription>
                {t(
                  "reset-password-this-password-reset-link-is-invalid-or-has-expired",
                )}
              </CardDescription>
            </CardHeader>
            <CardContent className="text-center">
              {error && (
                <div className="mb-4 p-3 rounded text-sm bg-destructive/10 text-destructive border border-destructive/20">
                  {error}
                </div>
              )}
              <Button
                onClick={() => navigate("/forgot-password")}
                variant="outline"
              >
                {t("reset-password-request-new-reset-link")}
              </Button>
            </CardContent>
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
              {t("reset-password-reset-your-password")}
            </CardTitle>
            <CardDescription>
              {t("reset-password-enter-your-new-password-below")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="password">
                  {t("reset-password-new-password")}
                </Label>
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
                  {t("reset-password-confirm-new-password")}
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
                  ? t("reset-password-updating")
                  : t("reset-password-update-password")}
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
