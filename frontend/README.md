# Frontend

[![Nuxt UI](https://img.shields.io/badge/Made%20with-Nuxt%20UI-00DC82?logo=nuxt&labelColor=020420)](https://ui.nuxt.com)

This is the frontend of the OJ system, built with [Nuxt](https://nuxt.com), [Nuxt UI](https://ui.nuxt.com), [Vue](https://vuejs.org), and [Vite](https://vitejs.dev).

> [!IMPORTANT]
> _**pnpm**_ is used for package management. You MUST use it.

## Nuxt Modules

This project uses the following Nuxt modules:

- [`@nuxt/devtools`](https://devtools.nuxt.com/)
- [`@nuxt/eslint`](https://eslint.nuxt.com/)
- [`@nuxt/test-utils`](https://nuxt.com/docs/4.x/getting-started/testing)
- [`@nuxt/image`](https://image.nuxt.com/)
- [`@nuxt/icon`](https://github.com/nuxt/icon)
- [`@nuxt/fonts`](https://fonts.nuxt.com/)
- [`@nuxt/ui`](https://ui.nuxt.com/)
- [`@nuxt/scripts`](https://scripts.nuxt.com/)
- [`@nuxt/content`](https://content.nuxt.com/)
- [`@nuxt/hints`](https://github.com/nuxt/hints)
- [`@nuxt/a11y`](https://github.com/nuxt/a11y)
- [`@nuxtjs/i18n`](https://i18n.nuxtjs.org/)
- [`@nuxtjs/color-mode`](https://color-mode.nuxtjs.org/)

## Setup

Make sure to install the dependencies:

```bash
pnpm install
```

## Development Server

Start the development server on `http://localhost:3000`:

```bash
pnpm dev
```

## Linting and Type Checking

Run ESLint to lint the code:

```bash
pnpm lint
```

Run Nuxt's type checker:

```bash
pnpm typecheck
```

## Testing

This project uses [Vitest](https://vitest.dev/) for testing. [Playwright](https://playwright.dev/) is used for end-to-end testing.

The test files are located in the `tests` directory, structured as follows:

```
tests/
|   # Vitest-powered unit tests
├── unit/
|   # Nuxt-specific tests using @nuxt/test-utils,
|   # integration test (for testing Nuxt components, routes, etc.)
├── nuxt/
|   # Playwright-powered end-to-end tests
└── e2e/
```

Run all tests:

```bash
pnpm test
```

Run unit tests:

```bash
pnpm test:unit
```

Run Nuxt tests:

```bash
pnpm test:nuxt
```

Run end-to-end tests:

```bash
pnpm test:e2e
```

Run end-to-end tests with Playwright UI:

```bash
pnpm test:e2e:ui
```

Run tests in watch mode:

```bash
pnpm test:watch
```

Get test coverage report:

```bash
pnpm test:coverage
```

## Production

Build the application for production:

```bash
pnpm build
```

Locally preview production build:

```bash
pnpm preview
```

Check out the [deployment documentation](https://nuxt.com/docs/getting-started/deployment) for more information.
