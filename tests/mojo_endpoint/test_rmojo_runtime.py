# -*- coding: utf-8 -*-
"""mojo_endpoint(runtime="rmojo"/"auto"): RMojo 実行系・上限・自動切替(rmojo_py が無ければ skip)。fastapi が無くても動く。"""
import sys, os, unittest
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "python"))
try:
    import rmojo_py  # noqa: F401
except ImportError:  # pragma: no cover
    rmojo_py = None
import importlib.util
_spec = importlib.util.spec_from_file_location("mf_mojo", os.path.join(os.path.dirname(__file__), "..", "..", "python", "mfastapi", "mojo.py"))
_m = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(_m)  # パッケージ __init__(starlette 依存)を経由せず読む
mojo_endpoint = _m.mojo_endpoint

SRC = "def fib(n: int) -> int:\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n\ndef spin(n: int) -> int:\n    i = 0\n    while True:\n        i = i + 1\n    return i\n"


class Resp:
    def __init__(self): self.headers = {}


@unittest.skipIf(rmojo_py is None, "rmojo_py not built")
class RmojoRuntime(unittest.TestCase):
    def test_rmojo(self):
        e = mojo_endpoint(None, {"n": int}, returns=int, runtime="rmojo", source=SRC, func="fib")
        r = Resp()
        self.assertEqual(e(n=20, _mf_response=r) if "_mf_response" in e.__signature__.parameters else e(n=20), 6765)

    def test_header_and_limit(self):
        e = mojo_endpoint(None, {"n": int}, returns=int, runtime="rmojo", source=SRC, func="fib", max_ms=200)
        r = Resp(); e(n=10, _mf_response=r)
        self.assertEqual(r.headers["x-mfastapi-runtime"], "rmojo")
        s = mojo_endpoint(None, {"n": int}, returns=int, runtime="rmojo", source=SRC, func="spin", max_ms=200)
        with self.assertRaises(Exception) as c:
            s(n=1)
        self.assertTrue("504" in repr(c.exception) or "LimitError" in str(c.exception))

    def test_auto_falls_back(self):
        def mojo_fib(n): raise ImportError("no mojo")
        e = mojo_endpoint(mojo_fib, {"n": int}, returns=int, runtime="auto", source=SRC, func="fib", name="fib")
        r = Resp(); self.assertEqual(e(n=15, _mf_response=r), 610)
        self.assertEqual(r.headers["x-mfastapi-runtime"], "rmojo")

    def test_mojo_used_when_available(self):
        e = mojo_endpoint(lambda n: n * 2, {"n": int}, returns=int, runtime="auto", source=SRC, func="fib", name="fib")
        r = Resp(); self.assertEqual(e(n=4, _mf_response=r), 8)
        self.assertEqual(r.headers["x-mfastapi-runtime"], "mojo")

    def test_bad_runtime(self):
        with self.assertRaises(ValueError): mojo_endpoint(None, {"n": int}, runtime="x")


if __name__ == "__main__":
    unittest.main()
