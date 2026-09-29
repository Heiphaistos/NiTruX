"""Functional scenarios on the real app.

Default scenarios are harmless (terminal, reports, benchmark, file tools,
drivers). `--privileged` adds the ones that CHANGE THE SYSTEM through
pkexec: activating the integration, installing then removing the `cowsay`
package, and a firewall cycle (enable, add/remove 8080/tcp, then restore
the initial on/off state). Each one asks for the admin password unless a
polkit agent/rule answers for you."""
import os, re, shutil, sys, tempfile, time
from selenium.webdriver.common.action_chains import ActionChains
from lib import start, goto, click, fill, text, wait_text, dangers, errs, shot

PRIVILEGED = "--privileged" in sys.argv
d = start()
results = []


def step(name, fn):
    try:
        res = fn()
        results.append((name, "OK" if res in (None, True) else f"ÉCHEC : {res}"))
    except Exception as e:  # noqa: BLE001 -- report and continue with the next scenario
        results.append((name, f"EXCEPTION : {e}"))
    js = errs(d)
    if js:
        results.append((f"{name} [erreurs JS]", str(js)[:400]))
    shot(d, f"flow_{name}")


def term_rows():
    return d.execute_script("return document.querySelector('.xterm-rows').innerText")


def terminal():
    goto(d, "Terminal", 2)
    d.execute_script("document.querySelector('.xterm-helper-textarea').focus()")
    # octal escapes: WebDriver cannot type non-ASCII, the shell prints "héllo"
    ActionChains(d).send_keys("printf 'h\\303\\251llo\\n'\n").perform()
    time.sleep(2)
    if "héllo" not in term_rows():
        return "accents mal décodés : " + term_rows()[-200:]
    ActionChains(d).send_keys("exit\n").perform()
    if not wait_text(d, "Nouvelle session", 5):
        return "pas de bouton « Nouvelle session » après exit"
    click(d, "Nouvelle session")
    time.sleep(2)
    ActionChains(d).send_keys("echo RESTARTED_OK\n").perform()
    time.sleep(1.5)
    return "RESTARTED_OK" in term_rows() or "la nouvelle session ne répond pas"


def report():
    goto(d, "Générateur de rapport")
    click(d, "Générer")
    if not wait_text(d, "Aperçu", 60):
        return "pas d'aperçu"
    click(d, "Télécharger en PDF")
    return wait_text(d, re.compile(r"rapport-\d+(-\d+)?\.pdf"), 60) or f"PDF non listé : {dangers(d)}"


def benchmark():
    goto(d, "Benchmark")
    click(d, "Lancer le benchmark")
    time.sleep(3)
    if not wait_text(d, "Lancer le benchmark", 300):
        return "benchmark jamais terminé"
    return not dangers(d) or str(dangers(d))


def file_tools():
    tmp = tempfile.mkdtemp(prefix="nitrux-e2e-")
    try:
        os.makedirs(f"{tmp}/sub")
        for p in (f"{tmp}/a.txt", f"{tmp}/sub/b.txt"):
            with open(p, "w") as f:
                f.write("même contenu\n")
        with open(f"{tmp}/big.bin", "wb") as f:
            f.write(os.urandom(2_000_000))
        goto(d, "Doublons / Gros fichiers / Hash")
        fill(d, "Dossier à scanner", tmp)
        click(d, "Rechercher")
        if not wait_text(d, "b.txt", 60):
            return "doublons non trouvés"
        click(d, "Gros fichiers")
        d.execute_script("const i=document.querySelectorAll('.nx-input');"
                         "i[0].value=arguments[0]; i[0].dispatchEvent(new Event('input'));"
                         "i[1].value='1,5'; i[1].dispatchEvent(new Event('input'));", tmp)
        click(d, "Rechercher")
        return wait_text(d, "big.bin", 60) or f"gros fichier non trouvé : {dangers(d)}"
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def drivers():
    goto(d, "Pilotes", 3)
    return "Périphériques & pilotes" in text(d) and not any("Impossible" in x for x in dangers(d)) or str(dangers(d))


def activate():
    if "Activer les fonctions privilégiées" not in text(d):
        return True  # already installed and up to date
    click(d, "Activer les fonctions privilégiées")
    return wait_text(d, "activées avec succès", 120) or f"échec : {text(d)[:300]}"


def package_cycle():
    goto(d, "Gestionnaire de paquets", 3)
    fill(d, "Nom du paquet à installer", "cowsay")
    click(d, "Installer")
    end = time.time() + 300
    while time.time() < end and not shutil.which("cowsay", path="/usr/games:/usr/bin"):
        time.sleep(2)
    if not shutil.which("cowsay", path="/usr/games:/usr/bin"):
        return f"cowsay non installé : {dangers(d)}"
    goto(d, "Désinstalleur", 4)
    fill(d, "Rechercher un paquet", "cowsay")
    time.sleep(1)
    click(d, "Désinstaller")
    fill(d, "pour confirmer", "cowsay")
    click(d, "Confirmer la désinstallation")
    end = time.time() + 300
    while time.time() < end and shutil.which("cowsay", path="/usr/games:/usr/bin"):
        time.sleep(2)
    return not shutil.which("cowsay", path="/usr/games:/usr/bin") or "cowsay toujours installé"


def firewall_cycle():
    goto(d, "Pare-feu", 3)
    if dangers(d):
        return f"erreur au chargement : {dangers(d)}"
    was_active = "UFW actif" in text(d)
    if not was_active:
        click(d, "Activer le pare-feu")
        click(d, "Confirmer l'activation")
        if not wait_text(d, "UFW actif", 60):
            return f"activation échouée : {dangers(d)}"
    else:
        click(d, "Afficher les règles", exact=False)
        time.sleep(5)
    fill(d, "8080/tcp", "8080/tcp")
    click(d, "Autoriser")
    if not wait_text(d, "8080/tcp ALLOW", 60):
        return f"règle non affichée : {dangers(d)}"
    click(d, "Retirer")
    time.sleep(6)
    if "8080/tcp ALLOW" in text(d):
        return "règle non retirée"
    if not was_active:
        click(d, "Désactiver le pare-feu")
        if not wait_text(d, "UFW inactif", 60):
            return "impossible de remettre le pare-feu inactif"
    return True


try:
    if PRIVILEGED:
        step("activation_privileges", activate)
    step("terminal", terminal)
    step("rapport_pdf", report)
    step("benchmark", benchmark)
    step("outils_fichiers", file_tools)
    step("pilotes", drivers)
    if PRIVILEGED:
        step("paquet_install_desinstall", package_cycle)
        step("pare_feu", firewall_cycle)
finally:
    d.quit()

for name, res in results:
    print(f"{'OK ' if res == 'OK' else '!! '} {name}: {res}")
sys.exit(0 if all(r == "OK" for _, r in results) else 1)
