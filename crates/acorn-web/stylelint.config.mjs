export default {
    extends: ["stylelint-config-recommended"],
    rules: {
        // Theme and component overrides intentionally use source order.
        "no-descending-specificity": null,
        "no-duplicate-selectors": null,
    },
};
