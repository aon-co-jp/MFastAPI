# -*- coding: utf-8 -*-
"""実 FastAPI アプリ越しに runtime="rmojo" と x-mfastapi-runtime ヘッダー・504 を確認(fastapi/httpx/rmojo_py が無ければ skip)。"""
import sys, os, unittest, importlib.util
try:
    import rmojo_py  # noqa: F401
    from fastapi import FastAPI
    from fastapi.testclient import TestClient
except ImportError:  # pragma: no cover
    rmojo_py = None
_p = os.path.join(os.path.dirname(__file__), "..", "..", "python", "mfastapi", "mojo.py")
SRC = "def fib(n: int) -> int:\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n\ndef spin(n: int) -> int:\n    i = 0\n    while True:\n        i = i + 1\n    return i\n"


@unittest.skipIf(rmojo_py is None, "fastapi/httpx/rmojo_py not available")
class ViaFastAPI(unittest.TestCase):
    def test_all(self):
        s = importlib.util.spec_from_file_location("mf_mojo", _p); m = importlib.util.module_from_spec(s); s.loader.exec_module(m)
        app = FastAPI()
        app.add_api_route("/fib/{n}", m.mojo_endpoint(None, {"n": int}, returns=int, runtime="rmojo", source=SRC, func="fib"), methods=["GET"])
        app.add_api_route("/spin/{n}", m.mojo_endpoint(None, {"n": int}, returns=int, runtime="rmojo", source=SRC, func="spin", max_ms=200), methods=["GET"])
        c = TestClient(app)
        r = c.get("/fib/20")
        self.assertEqual((r.status_code, r.json(), r.headers["x-mfastapi-runtime"]), (200, 6765, "rmojo"))
        self.assertEqual(c.get("/spin/1").status_code, 504)
        self.assertEqual(c.get("/fib/abc").status_code, 422)
        self.assertEqual(c.get("/fib/10").status_code, 200)  # 504 の後も復帰
        self.assertIn("/fib/{n}", c.get("/openapi.json").json()["paths"])


if __name__ == "__main__":
    unittest.main()
