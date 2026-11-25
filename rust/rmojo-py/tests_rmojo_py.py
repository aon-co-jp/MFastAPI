import sys, time, threading
sys.path.insert(0, sys.argv[1])
import rmojo_py as r
SRC = """
def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

def hello(s: str) -> str:
    return "hi " + s

def spin() -> int:
    i = 0
    while True:
        i = i + 1
    return i
"""
h = r.Handler(SRC, 50_000_000, 300)
assert h.call("fib", 20) == 6765
assert h.call("hello", "Taro") == "hi Taro"
try: h.call("spin"); raise SystemExit("spin did not stop")
except TimeoutError as e: print("timeout ok:", e)
try: h.call("nope"); raise SystemExit("no error")
except ValueError as e: print("value error ok:", e)
try: r.Handler("def f(:\n"); raise SystemExit("compile accepted")
except ValueError: print("compile error ok")
assert h.call("fib", 10) == 55  # worker recovered
# GIL release: a Python thread keeps ticking while a handler spins
ticks = []
stop = False
def ticker():
    while not stop: ticks.append(1); time.sleep(0.01)
t = threading.Thread(target=ticker); t.start()
try: h.call("spin")
except TimeoutError: pass
stop = True; t.join()
print("ticks during 300ms spin:", len(ticks)); assert len(ticks) > 10
print("ALL OK")
