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
|   # containing Nuxt-related unit tests and integration test
|   # (for testing Nuxt components, routes, etc.)
├── nuxt/
|   # Playwright-powered end-to-end tests
└── e2e/
```

The designated use of each test type is as follows:

- **Unit tests**: For testing individual functions, mostly logical ones that are not tightly coupled with Nuxt. These tests SHOULD be fast and cover a wide range of cases, including edge cases, and SHOULD NOT require any Nuxt-specific setup. They are ideal for testing utility functions, data processing logic, and other pure functions. These tests MUST NOT rely on the actual backend server and SHOULD use mocked API responses instead.
- **Nuxt tests**: For testing Nuxt components, routes, and other features that require a Nuxt environment. These tests MAY involve some integration testing but SHOULD NOT be as slow as end-to-end tests. They are ideal for testing the rendering of components, navigation between pages, and interactions that involve Nuxt features. These tests MUST NOT rely on the actual backend server and SHOULD use mocked API responses instead.
- **End-to-end tests**: For testing the entire application flow, including interactions with the backend server. These tests SHOULD be used sparingly and only for critical user flows that cannot be adequately tested with unit or Nuxt tests. They are ideal for testing key functionalities that involve user interaction (which can be simulated via Playwright), multiple components, and backend interactions. These tests MAY rely on the actual backend server, but it is RECOMMENDED to use mocked API responses to keep the tests fast and reliable.

>[!IMPORTANT]
> Note that the end-to-end tests (under `tests/e2e`) is configured to have a backend server compiled from the Rust code in the `backend` directory and ran before the tests are executed. Therefore, you do have the option to test against the actual backend server instead of a mocked one. **However**, you SHOULD NOT do that unless it is necessary, as it will significantly increase the time it takes to run the tests.

>[!NOTE]
> Note that the e2e tests are NOT tested in CI.

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
