export default {
    extends: ["stylelint-config-recommended"],
    // Preserve the archived LANL source stylesheet as exact brand-color evidence.
    ignoreFiles: ["**/assets/models/designs/laboratory-logos/los-alamos.css"],
    rules: {
        // Theme and component overrides intentionally use source order.
        "no-descending-specificity": null,
        "no-duplicate-selectors": null,
    },
};
