module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true,
  },
  extends: ["eslint:recommended"],
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
  },
  rules: {
    // 允許在開發和調試中使用 console
    "no-console": "off", // 暫時關閉 console 檢查，因為這些是調試用途
    // 其他規則
    "no-unused-vars": "warn",
    "no-undef": "error",
  },
  globals: {
    // Tauri 全域變數
    __TAURI__: "readonly",
    __TAURI_API__: "readonly",
    // 瀏覽器全域變數
    window: "readonly",
    document: "readonly",
    console: "readonly",
  },
};
