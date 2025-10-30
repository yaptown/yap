import {
  defineConfig,
  minimal2023Preset as preset,
} from "@vite-pwa/assets-generator/config";

export default defineConfig({
  headLinkOptions: {
    preset: "2023",
  },
  preset: {
    ...preset,
    apple: {
      sizes: [180],
      padding: 0.3,
      resizeOptions: {
        background: "#06003e",
        fit: "contain",
      },
    },
    maskable: {
      sizes: [512],
      padding: 0.3,
      resizeOptions: {
        background: "#06003e",
        fit: "contain",
      },
    },
  },
  images: ["public/yap.svg"],
});
