#!/usr/bin/env python3
"""
HFT Simulator visualizer.
Usage: python visualizer.py [path/to/binary]
Default binary: target/debug/HFTSimulator.exe
"""

import subprocess
import sys
import threading
import collections
import matplotlib.pyplot as plt
import matplotlib.animation as animation

MAX_POINTS = 300

ticks      = collections.deque(maxlen=MAX_POINTS)
best_bids  = collections.deque(maxlen=MAX_POINTS)
best_asks  = collections.deque(maxlen=MAX_POINTS)
mid_prices = collections.deque(maxlen=MAX_POINTS)
true_prices = collections.deque(maxlen=MAX_POINTS)
spreads    = collections.deque(maxlen=MAX_POINTS)

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


binary = sys.argv[1] if len(sys.argv) > 1 else r"target\debug\HFTSimulator.exe"
proc = subprocess.Popen(
   
    binary,          # string, not list
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
    bufsize=1,

)

reader = threading.Thread(target=read_output, args=(proc,), daemon=True)
reader.start()

fig, (ax_price, ax_spread) = plt.subplots(2, 1, figsize=(11, 7), sharex=True)
fig.suptitle("HFT Simulator", fontsize=13)


def update(_frame):
    with lock:
        t   = list(ticks)
        bid = list(best_bids)
        ask = list(best_asks)
        mid = list(mid_prices)
        true = list(true_prices)
        spr = list(spreads)

    ax_price.clear()
    ax_spread.clear()

    if t:
        ax_price.plot(t, mid,  label="Mid price",   color="#2196F3", linewidth=1.5)
        ax_price.plot(t, true, label="True price",  color="#FF9800", linewidth=1.5, linestyle="--")
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
plt.tight_layout()

try:
    plt.show()
finally:
    proc.terminate()
