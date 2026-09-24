"""Helpers shared by the NiTruX end-to-end scripts (WebDriver via tauri-driver)."""
import os, re, time
from pathlib import Path
from selenium import webdriver
from selenium.webdriver.common.options import ArgOptions
from selenium.webdriver.remote.client_config import ClientConfig

REPO = Path(__file__).resolve().parent.parent
APP = os.environ.get("NITRUX_APP", str(REPO / "src-tauri/target/debug/nitrux"))
OUT = Path(os.environ.get("NITRUX_E2E_OUT", REPO / "e2e/out"))
DRIVER = os.environ.get("TAURI_DRIVER_URL", "http://127.0.0.1:4444")

# Collects console.error, uncaught errors and unhandled promise rejections.
HOOK = """if(!window.__nx){window.__nx=[];const oe=console.error.bind(console);
console.error=(...a)=>{window.__nx.push('console.error: '+a.map(String).join(' '));oe(...a)};
window.addEventListener('error',e=>window.__nx.push('error: '+e.message));
window.addEventListener('unhandledrejection',e=>window.__nx.push('unhandled: '+String(e.reason)));}"""


def start():
    OUT.mkdir(parents=True, exist_ok=True)
    o = ArgOptions()
    o.set_capability("tauri:options", {"application": APP})
    o.set_capability("browserName", "wry")
    # Bounded: when the app dies at startup (no display, GTK failure) the
    # session request would otherwise wait forever.
    try:
        d = webdriver.Remote(options=o, client_config=ClientConfig(remote_server_addr=DRIVER, timeout=90))
    except Exception as e:
        log = REPO / "e2e/out-driver.log"
        tail = log.read_text(errors="replace").splitlines()[:3] if log.exists() else []
        raise SystemExit(f"L'application n'a pas démarré ({type(e).__name__}).\n" + "\n".join(tail))
    d.set_window_size(1400, 900)
    time.sleep(4)
    d.execute_script(HOOK)
    return d


def nav_labels(d):
    return d.execute_script("return [...document.querySelectorAll('.nx-app-nav button')].map(b => b.innerText.trim())")


def goto(d, label, wait=1.5):
    ok = d.execute_script(
        "const b=[...document.querySelectorAll('.nx-app-nav button')].find(b=>b.innerText.trim()===arguments[0]);"
        "if(!b) return false; b.scrollIntoView(); b.click(); return true;", label)
    assert ok, f"page introuvable dans le menu : {label}"
    time.sleep(wait)


def click(d, text, exact=True):
    ok = d.execute_script("""const t=arguments[0], ex=arguments[1];
      const b=[...document.querySelectorAll('button')].filter(b=>!b.closest('.nx-app-nav'))
        .find(b=>ex ? b.innerText.trim()===t : b.innerText.includes(t));
      if(!b) return false; b.scrollIntoView(); b.click(); return true;""", text, exact)
    assert ok, f"bouton introuvable : {text}"
    time.sleep(0.5)


def fill(d, placeholder, value):
    ok = d.execute_script("""const el=[...document.querySelectorAll('input,textarea')]
        .find(e=>(e.placeholder||e.getAttribute('aria-label')||'').includes(arguments[0]));
      if(!el) return false; el.value=arguments[1];
      el.dispatchEvent(new Event('input',{bubbles:true})); el.dispatchEvent(new Event('change',{bubbles:true}));
      return true;""", placeholder, value)
    assert ok, f"champ introuvable : {placeholder}"


def text(d):
    return d.execute_script("return document.body.innerText")


def wait_text(d, pattern, timeout=30):
    end = time.time() + timeout
    while time.time() < end:
        t = text(d)
        if (re.search(pattern, t) if isinstance(pattern, re.Pattern) else pattern in t):
            return True
        time.sleep(0.5)
    return False


def dangers(d):
    return d.execute_script("return [...document.querySelectorAll('.nx-card--danger')].map(e=>e.innerText.trim().slice(0,300))")


def errs(d):
    return d.execute_script("return window.__nx.splice(0)")


def shot(d, name):
    d.save_screenshot(str(OUT / f"{re.sub(r'[^A-Za-z0-9]+', '_', name)}.png"))
