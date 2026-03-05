import { expect, test } from '@nuxt/test-utils/playwright'

test('connect page shows the backend response', async ({ page, goto }) => {
  await goto('/connect', { waitUntil: 'hydration' })
  await expect(
    page.getByText('Hello frontend! This is Rust. Nou2X^mZwMq!4F$t')
  ).toBeVisible()
})
