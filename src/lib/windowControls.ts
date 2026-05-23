import { getCurrentWindow } from '@tauri-apps/api/window';

const isTauriRuntime = () => Boolean('__TAURI_INTERNALS__' in window);

export async function minimizeWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().toggleMaximize();
}

export async function hideWindow(): Promise<void> {
  if (!isTauriRuntime()) return;
  await getCurrentWindow().hide();
}
