#!/usr/bin/env python3
import subprocess
import sys
import threading
import collections
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib.widgets import Slider

PROJECT_DIR = r"C:\Users\nrper\CLionProjects\HFTSimulator"
BINARY = r"C:\Users\nrper\CLionProjects\HFTSimulator\target\debug\HFTSimulator.exe"

print("Compiling with gui feature...")
result = subprocess.run(
    ["cargo", "build", "--no-default-features", "--features", "gui"],
    cwd=PROJECT_DIR,
)
if result.returncode != 0:
    sys.exit("Compilation failed.")
print("Compilation succeeded.")

MAX_POINTS = 500

ticks       = collections.deque(maxlen=MAX_POINTS)
best_bids   = collections.deque(maxlen=MAX_POINTS)
best_asks   = collections.deque(maxlen=MAX_POINTS)
mid_prices  = collections.deque(maxlen=MAX_POINTS)
true_prices = collections.deque(maxlen=MAX_POINTS)
spreads     = collections.deque(maxlen=MAX_POINTS)

lock = threading.Lock()


def read_output(proc):
    for raw in proc.stdout:
        line = raw.strip()
        if not line.startswith("TICK:"):
            continue
        parts = line[5:].split(",")
        if len(parts) != 5:
            continue
        try:
            tick_n, bid, ask, mid, true = (int(p) for p in parts)
        except ValueError:
            continue
        with lock:
            ticks.append(tick_n)
            best_bids.append(bid)
            best_asks.append(ask)
            mid_prices.append(mid)
            true_prices.append(true)
            spreads.append(ask - bid)


proc = subprocess.Popen(
    [BINARY],
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
    bufsize=1,
)

reader = threading.Thread(target=read_output, args=(proc,), daemon=True)
reader.start()

fig, (ax_price, ax_spread) = plt.subplots(2, 1, figsize=(11, 8), sharex=True)
fig.suptitle("HFT Simulator", fontsize=13)
plt.subplots_adjust(bottom=0.17)

# Window slider: how many ticks to display at once
ax_window = fig.add_axes([0.15, 0.09, 0.65, 0.025])
window_slider = Slider(ax_window, "Window", 10, MAX_POINTS, valinit=100, valstep=10, color="#90CAF9")
ax_window.text(1.02, 0.5, "ticks", transform=ax_window.transAxes, va="center", fontsize=8)

# Pan slider: 0 = live (most recent), positive = scroll back into history
ax_pan = fig.add_axes([0.15, 0.04, 0.65, 0.025])
pan_slider = Slider(ax_pan, "Pan", 0, MAX_POINTS, valinit=0, valstep=1, color="#FFCC80")
ax_pan.text(1.02, 0.5, "← back", transform=ax_pan.transAxes, va="center", fontsize=8)


def update(_frame):
    with lock:
        t    = list(ticks)
        bid  = list(best_bids)
        ask  = list(best_asks)
        mid  = list(mid_prices)
        true = list(true_prices)
        spr  = list(spreads)

    n   = int(window_slider.val)
    pan = int(pan_slider.val)
    total = len(t)

    # end is offset from the tail; clamp so we always have at least `n` points
    end   = max(total - pan, n)
    end   = min(end, total)
    start = max(end - n, 0)

    t, bid, ask, mid, true, spr = (
        t[start:end], bid[start:end], ask[start:end],
        mid[start:end], true[start:end], spr[start:end],
    )

    ax_price.clear()
    ax_spread.clear()

    if t:
        ax_price.plot(t, mid,  label="Mid price",  color="#2196F3", linewidth=1.5)
        ax_price.plot(t, true, label="True price", color="#FF9800", linewidth=1.5, linestyle="--")
        ax_price.fill_between(t, bid, ask, alpha=0.15, color="#4CAF50", label="Bid-ask band")
        ax_price.set_ylabel("Price")
        ax_price.legend(loc="upper left", fontsize=8)
        ax_price.grid(True, alpha=0.3)

        ax_spread.plot(t, spr, color="#E91E63", linewidth=1.2)
        ax_spread.set_ylabel("Spread (ticks)")
        ax_spread.set_xlabel("Tick")
        ax_spread.grid(True, alpha=0.3)
        ax_spread.set_title("Bid-Ask Spread", fontsize=9)

    return []


ani = animation.FuncAnimation(fig, update, interval=150, cache_frame_data=False)
plt.tight_layout(rect=[0, 0.14, 1, 1])

try:
    plt.show()
finally:
    proc.terminate()
