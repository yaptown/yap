export const APP_STORE_URL = "https://apps.apple.com/app/yap-town/id6815057692";

/** iPhones, iPods, and iPads, including iPadOS Safari, which reports itself
 *  as a Mac but gives itself away with a touchscreen. */
export const IS_IOS =
  /iphone|ipad|ipod/i.test(navigator.userAgent) ||
  (/macintosh/i.test(navigator.userAgent) && navigator.maxTouchPoints > 1);
