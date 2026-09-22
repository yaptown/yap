import { get_challenge_restrictions } from "../../../yap-frontend-rs/pkg";

// Browser adapter shared by Review and the hub's scheduling views.
export function readChallengeRestrictions(timestampMs = Date.now()) {
  const read = (key: string) => {
    const raw = localStorage.getItem(key);
    return raw ? parseInt(raw, 10) : undefined;
  };
  const result = get_challenge_restrictions(
    read("yap-cant-listen-timestamp"),
    read("yap-cant-speak-timestamp"),
    timestampMs,
  );
  if (!result.banned.includes("Listening"))
    localStorage.removeItem("yap-cant-listen-timestamp");
  if (!result.banned.includes("Speaking"))
    localStorage.removeItem("yap-cant-speak-timestamp");
  return result;
}
