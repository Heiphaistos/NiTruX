// Automatic updates.
//
// Unlike Windows, Linux needs no code of our own here: the official Tauri
// updater knows all three shapes NiTruX ships as, and picks the right one from
// how the running copy was installed.
//
//   AppImage  the running file is renamed aside and the new one written in its
//             place — the portable, trace-free shape, and the only one that
//             needs no privileges at all
//   deb/rpm   dpkg -i / rpm, elevated through pkexec (then zenity/kdialog, then
//             terminal sudo)
//
// One manifest serves all three: the plugin looks for `linux-x86_64-appimage`,
// `-deb`, `-rpm`, then falls back to `linux-x86_64`.
//
// That password prompt on deb/rpm is the correct behaviour, not an annoyance to
// engineer away: a polkit rule allowing package installs without authentication
// would hand any local user a way to install arbitrary code as root. Whoever
// does not want the prompt runs the AppImage.

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { ask, message } from "@tauri-apps/plugin-dialog";

/**
 * Look for an update, offer it, install it and restart.
 *
 * @param silent true = say nothing when there is no update or the channel is
 *               unreachable (the check on startup); false = the "check for
 *               updates" button, which owes the user an answer either way.
 * @returns true if an update was installed.
 */
export async function checkForUpdate(silent = true): Promise<boolean> {
  // In dev the binary is not installed: there is nothing to replace.
  if (import.meta.env.DEV) return false;

  try {
    const update = await check();
    if (!update) {
      if (!silent) await message("NiTruX est à jour.", { title: "Mise à jour" });
      return false;
    }

    const notes = update.body ? `\n\n${update.body}` : "";
    const ok = await ask(
      `NiTruX ${update.version} est disponible (vous avez la ${update.currentVersion}).${notes}\n\n` +
        "Voulez-vous la mettre à jour maintenant ? L'application redémarrera.",
      { title: "Une nouvelle version est sortie", kind: "info" },
    );
    if (!ok) return false;

    await update.downloadAndInstall();
    // Unlike Windows — where the plugin exits the process itself so the
    // installer can replace the files — the Linux paths return here, so the
    // restart is ours to trigger.
    await relaunch();
    return true;
  } catch (e) {
    // Channel down, server unreachable, signature refused: the user keeps
    // working with the version they have.
    console.error("Vérification de mise à jour impossible", e);
    if (!silent) {
      await message(`Vérification impossible : ${e}`, { title: "Mise à jour", kind: "error" });
    }
    return false;
  }
}
