import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'

/**
 * Type-aware linting (recommendedTypeChecked) — the TypeScript checker
 * feeds ESLint, so rules like no-floating-promises / no-misused-promises
 * run with full type information. Project service is scoped to `src` so
 * tooling configs (vite/tsconfig.node) stay out of the lint graph.
 * لینت آگاه به نوع (recommendedTypeChecked) — چک‌کنندهٔ TypeScript به
 * ESLint خوراک می‌دهد تا قوانینی مانند no-floating-promises با اطلاعات
 * کامل نوع اجرا شوند. سرویس پروژه به `src` محدود است تا کانفیگ ابزارها
 * (vite/tsconfig.node) خارج از گراف لینت بماند.
 */
export default tseslint.config(
  { ignores: ['dist', 'coverage', 'src-tauri'] },
  {
    // Root-level build config is not part of the app's `tsconfig.json`
    // project; lint it with the non-type-aware rule set instead.
    // کانفیگ‌های سطح ریشه عضو پروژهٔ `tsconfig.json` برنامه نیستند؛ با
    // مجموعه‌قوانین غیر type-aware لینت می‌شوند.
    files: ['vite.config.ts'],
    extends: [js.configs.recommended, tseslint.configs.disableTypeChecked],
    languageOptions: {
      ecmaVersion: 2020,
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    extends: [
      js.configs.recommended,
      ...tseslint.configs.recommendedTypeChecked,
    ],
    // Build config and Playwright E2E specs live outside the `tsconfig.json`
    // project. `vite.config.ts` is linted by the dedicated non-type-aware
    // block above; the E2E specs get their own project + block below so they
    // never leak into the app's type-aware graph.
    // کانفیگ بیلد و specهای E2E خارج از پروژهٔ `tsconfig.json` هستند.
    // `vite.config.ts` در بلاک غیر type-aware بالا لینت می‌شود؛ specهای E2E
    // نیز پروژه و بلوک مخصوص خود را دارند تا هرگز وارد گراف آگاه به نوع برنامه نشوند.
    ignores: ['vite.config.ts', 'tests/e2e/**'],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        project: ['./tsconfig.json'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // Test files deliberately use non-null assertions on mock arguments;
      // banning them globally would only add noise to already-covered code.
      // فایل‌های تست آگاهانه از assertion غیر-null روی آرگومان‌های mock
      // استفاده می‌کنند؛ ممنوعیت سراسری فقط نویز اضافه می‌کند.
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  },
  {
    // Playwright E2E specs run against the release binary in a Node context,
    // so they use a dedicated tsconfig project and Node-aware globals. This
    // block is type-aware within that project only.
    // specهای E2E در بافت Node و علیه باینری انتشار اجرا می‌شوند؛ بنابراین از
    // پروژهٔ tsconfig اختصاصی و globals مخصوص Node استفاده می‌کنند. این بلوک
    // فقط در محدودهٔ همان پروژه، آگاه به نوع است.
    files: ['tests/e2e/**/*.{ts,tsx}'],
    extends: [js.configs.recommended, ...tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      ecmaVersion: 2022,
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        project: ['./tests/e2e/tsconfig.json'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      // Playwright idioms: test callbacks commonly declare `async` and
      // destructure a fixture (e.g. `{ page }`) even when the body needs
      // neither — e.g. to assert on globals or skip. `args: 'none'` stops
      // the unused-arg check from flagging these, and `require-await` is
      // off because a callback may intentionally be sync-shaped.
      // الگوهای Playwright: callback تست معمولاً `async` است و fixture
      // (مثلاً `{ page }`) را باز می‌کند حتی وقتی بدنه به آن‌ها نیاز ندارد —
      // مثلاً برای assert روی globals یا skip. `args: 'none'` بررسی آرگومان
      // استفاده‌نشده را خاموش می‌کند و `require-await` هم آفلاین است چون
      // ممکن است callback عمداً sync‌شکل باشد.
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', args: 'none' }],
      '@typescript-eslint/require-await': 'off',
      // E2E helpers intentionally assert on the just-spawned process handle.
      // عدم بکارگیری assertion غیر-null صرفاً برای این فایل‌ها مجاز است.
      '@typescript-eslint/no-non-null-assertion': 'off',
    },
  }
)
