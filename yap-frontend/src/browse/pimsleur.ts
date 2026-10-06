const KEY = "yap-pimsleur-acknowledged";

/** Whether the learner has read that Yap isn't affiliated with Pimsleur. */
export function pimsleurAcknowledged() {
  return localStorage.getItem(KEY) === "true";
}

export function acknowledgePimsleur() {
  localStorage.setItem(KEY, "true");
}
