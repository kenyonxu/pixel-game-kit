import type { UiSchema } from "@rjsf/utils";

export const uiSchema: UiSchema = {
  "ui:order": [
    "k_colors",
    "pixel_size_override",
    "detect_strategy",
    "resample_method",
    "colorspace",
    "dither",
    "preset_palette",
    "palette",
    "postprocess",
  ],
  k_colors: {
    "ui:widget": "range",
    "ui:options": {
      label: false,
    },
  },
  pixel_size_override: {
    "ui:placeholder": "auto",
  },
  detect_strategy: {
    "ui:widget": "select",
  },
  resample_method: {
    "ui:widget": "select",
  },
  colorspace: {
    "ui:widget": "select",
  },
  dither: {
    "ui:widget": "select",
  },
  preset_palette: {
    "ui:widget": "select",
  },
  palette: {
    "ui:options": {
      orderable: false,
    },
  },
  postprocess: {
    bg_remove: {
      "ui:widget": "checkboxes",
    },
    bg_tolerance: {
      "ui:widget": "range",
      "ui:options": {
        label: false,
      },
    },
    bg_connectivity: {
      "ui:widget": "select",
    },
    bg_scope: {
      "ui:widget": "select",
    },
    bg_floating_threshold: {
      "ui:placeholder": "0",
    },
    outline: {
      "ui:widget": "select",
    },
    outline_color: {
      "ui:placeholder": "000000",
    },
    morph: {
      "ui:widget": "checkboxes",
    },
    alpha_threshold: {
      "ui:placeholder": '"" = off, "auto" = Otsu, or 0–255',
    },
  },
};
