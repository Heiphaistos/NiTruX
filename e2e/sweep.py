"""Opens every page of the menu, one after the other, and reports error
cards and JavaScript errors. Read-only: changes nothing on the system.
Screenshots and results.json go to e2e/out/."""
import json, os, sys, time
from lib import start, nav_labels, goto, dangers, errs, shot, OUT

wait = float(os.environ.get("WAIT", "4"))
d = start()
results, js_failures = [], 0
try:
    for i, label in enumerate(nav_labels(d)):
        goto(d, label, wait)
        js = errs(d)
        # The startup update check logs when the update server is
        # unreachable (offline VM, filtered network): expected, not a bug.
        info = {
            "page": label,
            "dangers": dangers(d),
            "js_errors": [e for e in js if "Vérification de mise à jour" not in e],
            "info": [e for e in js if "Vérification de mise à jour" in e],
        }
        for x in info["info"]:
            print(f"  (info) {x[:200]}", flush=True)
        shot(d, f"{i:02d}_{label}")
        results.append(info)
        js_failures += len(info["js_errors"])
        status = "OK" if not info["dangers"] and not info["js_errors"] else "!!"
        print(f"[{status}] {label}", flush=True)
        for x in info["dangers"] + info["js_errors"]:
            print("      " + x.replace("\n", " | ")[:300], flush=True)
finally:
    d.quit()
    (OUT / "sweep.json").write_text(json.dumps(results, ensure_ascii=False, indent=1))
print(f"\n{len(results)} pages, {js_failures} erreur(s) JavaScript. Captures : {OUT}")
sys.exit(1 if js_failures else 0)
