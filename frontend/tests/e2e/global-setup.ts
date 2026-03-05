import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

export default function () {
  const rootDir = path.resolve(fileURLToPath(new URL('../../', import.meta.url)))
  execSync('pnpm nuxt build', { cwd: rootDir, stdio: 'inherit' })
}
