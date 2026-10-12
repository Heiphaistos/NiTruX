// A command another page wants typed into the integrated terminal: updates
// that must run as the user (omarchy-update, yay) or interactive tools
// (photorec, fwupdmgr). The page sets it, then navigates to "terminal",
// which types it once its shell is up. The user sees and can stop it.
import { ref } from "vue";

export const pendingTerminalCommand = ref<string | null>(null);

export function takePendingTerminalCommand(): string | null {
  const command = pendingTerminalCommand.value;
  pendingTerminalCommand.value = null;
  return command;
}
